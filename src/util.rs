//! Pequenos utilitários de formatação (dinheiro, datas).

use chrono::{Datelike, Local, NaiveDate};

/// Data de hoje no formato usado pelo banco (`AAAA-MM-DD`).
pub fn today_iso() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

/// Prefixo do mês atual (`AAAA-MM`), usado para somar recebimentos.
pub fn current_month_prefix() -> String {
    Local::now().format("%Y-%m").to_string()
}

/// Data de hoje no formato brasileiro.
pub fn today_br() -> String {
    Local::now().format("%d/%m/%Y").to_string()
}

/// Data por extenso curta, em português, para o cabeçalho.
pub fn today_long_br() -> String {
    const DIAS: [&str; 7] = [
        "segunda-feira",
        "terça-feira",
        "quarta-feira",
        "quinta-feira",
        "sexta-feira",
        "sábado",
        "domingo",
    ];
    let now = Local::now();
    let dia = DIAS[now.weekday().num_days_from_monday() as usize];
    format!("{}, {}", dia, now.format("%d/%m/%Y"))
}

/// Converte `AAAA-MM-DD` em `DD/MM/AAAA`.
pub fn iso_to_br(iso: &str) -> String {
    NaiveDate::parse_from_str(iso, "%Y-%m-%d")
        .map(|d| d.format("%d/%m/%Y").to_string())
        .unwrap_or_else(|_| iso.to_string())
}

/// Converte uma data digitada em `DD/MM/AAAA` para `AAAA-MM-DD`.
pub fn br_to_iso(input: &str) -> Option<String> {
    let s = input.trim();
    for fmt in ["%d/%m/%Y", "%d/%m/%y", "%Y-%m-%d"] {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Some(d.format("%Y-%m-%d").to_string());
        }
    }
    None
}

/// Converte texto digitado em centavos.
///
/// Aceita `120`, `120,50`, `1.234,56`, `R$ 1.234,56` e `120.50`.
pub fn parse_money(input: &str) -> Option<i64> {
    let cleaned: String = input
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == ',' || *c == '.' || *c == '-')
        .collect();
    let s = cleaned.trim();
    if s.is_empty() {
        return Some(0);
    }
    let negative = s.starts_with('-');
    let s = s.trim_start_matches('-');
    if s.is_empty() {
        return None;
    }

    let normalized = if s.contains(',') {
        s.replace('.', "").replace(',', ".")
    } else if s.matches('.').count() > 1 {
        s.replace('.', "")
    } else if s.contains('.') {
        let frac_len = s.split('.').nth(1).map(|p| p.len()).unwrap_or(0);
        if frac_len == 3 {
            s.replace('.', "")
        } else {
            s.to_string()
        }
    } else {
        s.to_string()
    };

    let value: f64 = normalized.parse().ok()?;
    if !value.is_finite() {
        return None;
    }
    let cents = (value * 100.0).round() as i64;
    Some(if negative { -cents } else { cents })
}

/// Formata centavos como `R$ 1.234,56`.
pub fn money(cents: i64) -> String {
    let negative = cents < 0;
    let abs = cents.unsigned_abs();
    let reais = abs / 100;
    let centavos = abs % 100;

    let digits = reais.to_string();
    let len = digits.len();
    let mut grouped = String::with_capacity(len + len / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            grouped.push('.');
        }
        grouped.push(ch);
    }

    format!(
        "{}R$ {},{:02}",
        if negative { "-" } else { "" },
        grouped,
        centavos
    )
}

/// Formata centavos sem o prefixo `R$`.
pub fn money_plain(cents: i64) -> String {
    money(cents).trim_start_matches("R$ ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_round_trip() {
        assert_eq!(parse_money("120,50"), Some(12050));
        assert_eq!(parse_money("R$ 1.234,56"), Some(123456));
        assert_eq!(parse_money("1.234"), Some(123400));
        assert_eq!(parse_money(""), Some(0));
        assert_eq!(money(123456), "R$ 1.234,56");
        assert_eq!(money(50), "R$ 0,50");
        assert_eq!(money(-50), "-R$ 0,50");
    }

    #[test]
    fn dates() {
        assert_eq!(br_to_iso("13/11/2026").as_deref(), Some("2026-11-13"));
        assert_eq!(iso_to_br("2026-11-13"), "13/11/2026");
        assert_eq!(br_to_iso("31/02/2026"), None);
    }
}
