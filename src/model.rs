//! Estruturas de domínio da oficina.

/// Cliente atendido pela oficina.
#[derive(Clone, Debug)]
pub struct Client {
    pub id: i64,
    pub name: String,
    pub phone: String,
    pub email: String,
    pub address: String,
    pub notes: String,
    pub created_at: String,
    pub equipment_count: i64,
}

/// Equipamento deixado para conserto.
#[derive(Clone, Debug)]
pub struct Equipment {
    pub id: i64,
    pub client_id: i64,
    pub kind: String,
    pub brand: String,
    pub model: String,
    pub serial: String,
    pub notes: String,
    pub created_at: String,
    pub client_name: String,
    pub quote_count: i64,
}

impl Equipment {
    pub fn title(&self) -> String {
        equipment_label(&self.kind, &self.brand, &self.model)
    }
}

pub fn equipment_label(kind: &str, brand: &str, model: &str) -> String {
    let kind = kind.trim();
    let brand = brand.trim();
    let model = model.trim();
    match (kind.is_empty(), brand.is_empty(), model.is_empty()) {
        (false, false, false) => format!("{kind} {brand} {model}"),
        (false, false, true) => format!("{kind} {brand}"),
        (false, true, false) => format!("{kind} {model}"),
        (false, true, true) => kind.to_string(),
        (true, false, false) => format!("{brand} {model}"),
        (true, false, true) => brand.to_string(),
        (true, true, false) => model.to_string(),
        (true, true, true) => "Equipamento".to_string(),
    }
}

/// Situação de um orçamento.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuoteStatus {
    Pendente,
    Aprovado,
    Concluido,
    Recusado,
}

impl QuoteStatus {
    pub const ALL: [QuoteStatus; 4] = [
        QuoteStatus::Pendente,
        QuoteStatus::Aprovado,
        QuoteStatus::Concluido,
        QuoteStatus::Recusado,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            QuoteStatus::Pendente => "pendente",
            QuoteStatus::Aprovado => "aprovado",
            QuoteStatus::Concluido => "concluido",
            QuoteStatus::Recusado => "recusado",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            QuoteStatus::Pendente => "Pendente",
            QuoteStatus::Aprovado => "Aprovado",
            QuoteStatus::Concluido => "Concluído",
            QuoteStatus::Recusado => "Recusado",
        }
    }

    pub fn parse(value: &str) -> QuoteStatus {
        match value {
            "aprovado" => QuoteStatus::Aprovado,
            "concluido" | "concluído" => QuoteStatus::Concluido,
            "recusado" => QuoteStatus::Recusado,
            _ => QuoteStatus::Pendente,
        }
    }

    /// Orçamentos que ainda representam trabalho em andamento.
    pub fn is_open(self) -> bool {
        matches!(self, QuoteStatus::Pendente | QuoteStatus::Aprovado)
    }
}

/// Orçamento de conserto, já com dados derivados do banco.
#[derive(Clone, Debug)]
pub struct Quote {
    pub id: i64,
    pub client_id: i64,
    pub equipment_id: Option<i64>,
    pub description: String,
    pub status: QuoteStatus,
    pub labor_cents: i64,
    pub parts_cents: i64,
    pub discount_cents: i64,
    pub created_at: String,
    pub client_name: String,
    pub equipment_label: String,
    pub paid_cents: i64,
}

impl Quote {
    pub fn total_cents(&self) -> i64 {
        (self.labor_cents + self.parts_cents - self.discount_cents).max(0)
    }

    pub fn balance_cents(&self) -> i64 {
        (self.total_cents() - self.paid_cents).max(0)
    }

    pub fn is_paid(&self) -> bool {
        self.total_cents() > 0 && self.balance_cents() == 0
    }

    pub fn summary(&self) -> String {
        let text = self.description.trim();
        if text.is_empty() {
            "Sem descrição".to_string()
        } else {
            text.to_string()
        }
    }
}

/// Pagamento recebido de um orçamento.
#[derive(Clone, Debug)]
pub struct Payment {
    pub id: i64,
    pub quote_id: i64,
    pub amount_cents: i64,
    pub method: String,
    pub paid_on: String,
    pub notes: String,
    pub client_name: String,
    pub quote_description: String,
    pub quote_total_cents: i64,
}

/// Números exibidos no painel.
#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub clients: usize,
    pub equipment: usize,
    pub open_quotes: usize,
    pub payable_quotes: usize,
    pub receivable_cents: i64,
    pub month_revenue_cents: i64,
    pub total_revenue_cents: i64,
}
