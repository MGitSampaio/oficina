//! Geração do documento de orçamento para impressão.
//!
//! O documento é escrito como HTML e aberto no navegador padrão, que já abre o
//! diálogo de impressão do sistema (o mesmo vale para "Salvar como PDF").

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::model::{Client, Equipment, Payment, Quote};
use crate::util::{iso_to_br, money, today_br};

/// Gera o HTML do orçamento e abre no aplicativo padrão do sistema.
pub fn quote(
    quote: &Quote,
    client: Option<&Client>,
    equipment: Option<&Equipment>,
    payments: &[Payment],
) -> Result<PathBuf, String> {
    let path = write_quote(quote, client, equipment, payments)?;
    open(&path)?;
    Ok(path)
}

/// Escreve o HTML em um arquivo temporário e devolve o caminho.
pub fn write_quote(
    quote: &Quote,
    client: Option<&Client>,
    equipment: Option<&Equipment>,
    payments: &[Payment],
) -> Result<PathBuf, String> {
    let html = quote_html(quote, client, equipment, payments);
    let path = std::env::temp_dir().join(format!("oficina-orcamento-{}.html", quote.id));
    std::fs::write(&path, html)
        .map_err(|err| format!("Não foi possível preparar o arquivo de impressão: {err}"))?;
    Ok(path)
}

fn open(path: &Path) -> Result<(), String> {
    let mut command = if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "start", ""]).arg(path);
        cmd
    } else if cfg!(target_os = "macos") {
        let mut cmd = Command::new("open");
        cmd.arg(path);
        cmd
    } else {
        let mut cmd = Command::new("xdg-open");
        cmd.arg(path);
        cmd
    };

    command
        .spawn()
        .map(|_| ())
        .map_err(|err| format!("Não foi possível abrir o navegador para impressão: {err}"))
}

fn esc(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn lines(value: &str) -> String {
    esc(value.trim()).replace('\n', "<br>")
}

fn status_class(quote: &Quote) -> &'static str {
    match quote.status {
        crate::model::QuoteStatus::Pendente => "pendente",
        crate::model::QuoteStatus::Aprovado => "aprovado",
        crate::model::QuoteStatus::Concluido => "concluido",
        crate::model::QuoteStatus::Recusado => "recusado",
    }
}

fn payment_rows(payments: &[Payment]) -> String {
    if payments.is_empty() {
        return "<tr><td colspan=\"3\" class=\"empty\">Nenhum pagamento registrado.</td></tr>"
            .to_string();
    }

    payments
        .iter()
        .map(|payment| {
            let notes = if payment.notes.trim().is_empty() {
                String::new()
            } else {
                format!("<br><span class=\"note\">{}</span>", lines(&payment.notes))
            };
            format!(
                "<tr><td>{}{}</td><td>{}</td><td class=\"right\">{}</td></tr>",
                esc(&iso_to_br(&payment.paid_on)),
                notes,
                esc(&payment.method),
                esc(&money(payment.amount_cents))
            )
        })
        .collect()
}

/// Monta o documento completo, pronto para impressão em A4.
pub fn quote_html(
    quote: &Quote,
    client: Option<&Client>,
    equipment: Option<&Equipment>,
    payments: &[Payment],
) -> String {
    let contact = client
        .map(|client| {
            let mut parts = Vec::new();
            if !client.phone.trim().is_empty() {
                parts.push(esc(&client.phone));
            }
            if !client.email.trim().is_empty() {
                parts.push(esc(&client.email));
            }
            if parts.is_empty() {
                String::new()
            } else {
                format!("<div class=\"muted\">{}</div>", parts.join(" · "))
            }
        })
        .unwrap_or_default();

    let address = client
        .filter(|client| !client.address.trim().is_empty())
        .map(|client| format!("<div class=\"muted\">{}</div>", lines(&client.address)))
        .unwrap_or_default();

    let equipment_title = if quote.equipment_label.trim().is_empty() {
        "Serviço avulso (sem equipamento)".to_string()
    } else {
        esc(&quote.equipment_label)
    };

    let serial = equipment
        .filter(|equipment| !equipment.serial.trim().is_empty())
        .map(|equipment| {
            format!(
                "<div class=\"muted\">Nº de série: {}</div>",
                esc(&equipment.serial)
            )
        })
        .unwrap_or_default();

    let description = if quote.description.trim().is_empty() {
        "Sem descrição detalhada.".to_string()
    } else {
        lines(&quote.description)
    };

    let paid_label = if quote.balance_cents() == 0 && quote.total_cents() > 0 {
        format!(
            "{} <span class=\"stamp\">Quitado</span>",
            money(quote.paid_cents)
        )
    } else {
        money(quote.paid_cents)
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="pt-BR">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Orçamento #{id} — Oficina</title>
<style>
  :root {{ color-scheme: light; }}
  * {{ box-sizing: border-box; }}
  body {{
    margin: 0; padding: 32px 16px 64px;
    font-family: "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    background: #f2f5f9; color: #151f30; font-size: 13px; line-height: 1.45;
  }}
  .sheet {{
    width: 210mm; min-height: 297mm; margin: 0 auto; padding: 18mm;
    background: #fff; border: 1px solid #e2e8f0; border-radius: 4px;
    box-shadow: 0 18px 50px rgba(15, 23, 42, .14);
  }}
  .top {{ display: flex; justify-content: space-between; align-items: flex-start; gap: 24px; }}
  .brand {{ display: flex; gap: 12px; align-items: center; }}
  .logo {{
    width: 44px; height: 44px; border-radius: 11px; background: #2563eb;
    color: #fff; font-size: 23px; font-weight: 700;
    display: flex; align-items: center; justify-content: center;
  }}
  h1 {{ font-size: 18px; margin: 0; letter-spacing: -.01em; }}
  .sub {{ font-size: 11px; color: #6b7a90; }}
  .doc {{ text-align: right; }}
  .doc .kicker {{
    font-size: 10px; font-weight: 700; letter-spacing: .14em;
    text-transform: uppercase; color: #94a3b8;
  }}
  .doc .number {{ font-size: 26px; font-weight: 700; letter-spacing: -.02em; }}
  .doc .date {{ font-size: 11px; color: #6b7a90; }}
  .status {{
    display: inline-block; margin-top: 7px; padding: 3px 11px; border-radius: 999px;
    font-size: 10.5px; font-weight: 700; text-transform: uppercase; letter-spacing: .05em;
  }}
  .pendente {{ color: #b45309; background: #fef3c7; }}
  .aprovado {{ color: #1d4ed8; background: #dbeafe; }}
  .concluido {{ color: #15803d; background: #dcfce7; }}
  .recusado {{ color: #b91c1c; background: #fee2e2; }}
  .rule {{ height: 1px; background: #e2e8f0; margin: 18px 0; }}
  .cols {{ display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }}
  .box {{ border: 1px solid #e2e8f0; border-radius: 8px; padding: 12px 14px; background: #f8fafc; }}
  .label {{
    font-size: 9.5px; font-weight: 700; letter-spacing: .11em;
    text-transform: uppercase; color: #94a3b8; margin-bottom: 4px;
  }}
  .name {{ font-size: 14px; font-weight: 600; }}
  .muted {{ font-size: 11.5px; color: #6b7a90; }}
  h2 {{
    font-size: 10px; font-weight: 700; letter-spacing: .12em; text-transform: uppercase;
    color: #94a3b8; margin: 22px 0 7px;
  }}
  .description {{ white-space: normal; }}
  table {{ width: 100%; border-collapse: collapse; margin-top: 4px; }}
  th {{
    font-size: 9.5px; font-weight: 700; letter-spacing: .08em; text-transform: uppercase;
    color: #94a3b8; text-align: left; padding: 7px 8px; border-bottom: 1px solid #e2e8f0;
  }}
  td {{ padding: 8px; border-bottom: 1px solid #eef2f7; vertical-align: top; }}
  .right {{ text-align: right; }}
  .total-row td {{ border-top: 2px solid #e2e8f0; font-weight: 700; font-size: 15px; }}
  .totals {{ margin-left: auto; width: 62mm; margin-top: 10px; }}
  .totals .line {{ display: flex; justify-content: space-between; padding: 4px 0; }}
  .totals .balance {{ font-size: 15px; font-weight: 700; color: #b45309; }}
  .stamp {{ color: #15803d; font-weight: 700; font-size: 11px; text-transform: uppercase; }}
  .empty {{ color: #94a3b8; text-align: center; padding: 14px; }}
  .note {{ font-size: 10px; color: #94a3b8; }}
  .signatures {{ display: grid; grid-template-columns: 1fr 1fr; gap: 24mm; margin-top: 26mm; }}
  .signatures div {{ border-top: 1px solid #94a3b8; padding-top: 5px; text-align: center; font-size: 10.5px; color: #6b7a90; }}
  .footer {{ margin-top: 14mm; border-top: 1px solid #e2e8f0; padding-top: 6px; font-size: 9.5px; color: #94a3b8; display: flex; justify-content: space-between; }}
  .toolbar {{
    width: 210mm; margin: 0 auto 10px; display: flex; justify-content: flex-end; gap: 8px;
  }}
  .toolbar button {{
    font: inherit; cursor: pointer; border: 0; border-radius: 8px; padding: 9px 16px;
    background: #2563eb; color: #fff; font-weight: 600; font-size: 12px;
  }}
  .toolbar .ghost {{ background: #fff; color: #151f30; border: 1px solid #e2e8f0; }}
  @media print {{
    body {{ background: #fff; padding: 0; font-size: 11.5px; }}
    .sheet {{ width: auto; min-height: 0; border: 0; border-radius: 0; box-shadow: none; padding: 0; }}
    .toolbar {{ display: none; }}
    @page {{ size: A4; margin: 14mm; }}
  }}
</style>
</head>
<body>
  <div class="toolbar">
    <button onclick="window.print()">Imprimir / Salvar em PDF</button>
    <button class="ghost" onclick="window.close()">Fechar</button>
  </div>

  <main class="sheet">
    <div class="top">
      <div class="brand">
        <div class="logo">O</div>
        <div>
          <h1>Oficina</h1>
          <div class="sub">Assistência técnica de eletrodomésticos</div>
        </div>
      </div>
      <div class="doc">
        <div class="kicker">Orçamento</div>
        <div class="number">#{id}</div>
        <div class="date">Emitido em {today}</div>
        <span class="status {status_class}">{status}</span>
      </div>
    </div>

    <div class="rule"></div>

    <div class="cols">
      <div class="box">
        <div class="label">Cliente</div>
        <div class="name">{client_name}</div>
        {contact}
        {address}
      </div>
      <div class="box">
        <div class="label">Equipamento</div>
        <div class="name">{equipment_title}</div>
        {serial}
      </div>
    </div>

    <h2>Descrição do serviço</h2>
    <div class="description">{description}</div>

    <h2>Valores</h2>
    <table>
      <thead><tr><th>Item</th><th class="right">Valor</th></tr></thead>
      <tbody>
        <tr><td>Mão de obra</td><td class="right">{labor}</td></tr>
        <tr><td>Peças</td><td class="right">{parts}</td></tr>
        <tr><td>Desconto</td><td class="right">− {discount}</td></tr>
        <tr class="total-row"><td>Total do orçamento</td><td class="right">{total}</td></tr>
      </tbody>
    </table>

    <div class="totals">
      <div class="line"><span>Pago</span><span>{paid}</span></div>
      <div class="line balance"><span>Saldo em aberto</span><span>{balance}</span></div>
    </div>

    <h2>Pagamentos</h2>
    <table>
      <thead><tr><th>Data / observação</th><th>Forma</th><th class="right">Valor</th></tr></thead>
      <tbody>{payments}</tbody>
    </table>

    <div class="signatures">
      <div>Assinatura do cliente</div>
      <div>Oficina</div>
    </div>

    <div class="footer">
      <span>Orçamento #{id} · emitido em {today}</span>
      <span>Oficina — gestão de assistência técnica</span>
    </div>
  </main>

  <script>
    window.addEventListener('load', function () {{
      setTimeout(function () {{ window.print(); }}, 350);
    }});
  </script>
</body>
</html>
"#,
        id = quote.id,
        today = today_br(),
        status_class = status_class(quote),
        status = quote.status.label(),
        client_name = esc(&quote.client_name),
        contact = contact,
        address = address,
        equipment_title = equipment_title,
        serial = serial,
        description = description,
        labor = money(quote.labor_cents),
        parts = money(quote.parts_cents),
        discount = money(quote.discount_cents),
        total = money(quote.total_cents()),
        paid = paid_label,
        balance = money(quote.balance_cents()),
        payments = payment_rows(payments),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Client, Quote, QuoteStatus};

    fn sample_quote() -> Quote {
        Quote {
            id: 42,
            client_id: 1,
            equipment_id: None,
            description: "Troca <da> resistência & teste <b>final</b>".to_string(),
            status: QuoteStatus::Aprovado,
            labor_cents: 12000,
            parts_cents: 8950,
            discount_cents: 1000,
            created_at: "2026-11-13".to_string(),
            client_name: "Maria & Filhos".to_string(),
            equipment_label: "Fogão Brastemp".to_string(),
            paid_cents: 5000,
        }
    }

    #[test]
    fn escapes_user_content() {
        let html = quote_html(&sample_quote(), None, None, &[]);
        assert!(html.contains("Maria &amp; Filhos"));
        assert!(html.contains("Troca &lt;da&gt; resistência &amp; teste &lt;b&gt;final&lt;/b&gt;"));
        assert!(!html.contains("<b>final</b>"));
    }

    #[test]
    fn includes_values_and_balance() {
        let quote = sample_quote();
        let html = quote_html(&quote, None, None, &[]);
        assert!(html.contains(&money(quote.total_cents())));
        assert!(html.contains(&money(quote.balance_cents())));
        assert!(html.contains("Nenhum pagamento registrado"));
        assert!(html.contains("window.print()"));
    }

    #[test]
    fn writes_html_file() {
        let path = write_quote(&sample_quote(), None, None, &[]).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("<!DOCTYPE html>"));
        assert!(content.contains("Orçamento #42"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn includes_client_and_equipment_details() {
        let client = Client {
            id: 1,
            name: "Maria & Filhos".to_string(),
            phone: "(11) 99999-0000".to_string(),
            email: "maria@example.com".to_string(),
            address: "Rua A, 10".to_string(),
            notes: String::new(),
            created_at: "2026-11-13".to_string(),
            equipment_count: 1,
        };
        let html = quote_html(&sample_quote(), Some(&client), None, &[]);
        assert!(html.contains("(11) 99999-0000"));
        assert!(html.contains("Rua A, 10"));
    }
}
