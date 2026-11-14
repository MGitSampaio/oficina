//! Formulários (janelas modais) de cadastro e edição.

use egui::{Align, CornerRadius, Frame, Layout, Margin, RichText, Ui};

use crate::db::{DbResult, Store};
use crate::model::{Client, Equipment, Payment, Quote, QuoteStatus};
use crate::theme::*;
use crate::ui::widgets as w;
use crate::util::{br_to_iso, iso_to_br, money, money_plain, parse_money, today_br};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogOutcome {
    Keep,
    Save,
    Cancel,
}

pub const PAYMENT_METHODS: [&str; 7] = [
    "Pix",
    "Dinheiro",
    "Cartão de débito",
    "Cartão de crédito",
    "Transferência",
    "Boleto",
    "Outro",
];

fn footer(ui: &mut Ui) -> DialogOutcome {
    let mut outcome = DialogOutcome::Keep;
    ui.horizontal(|ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.add(w::primary_button("Salvar")).clicked() {
                outcome = DialogOutcome::Save;
            }
            ui.add_space(4.0);
            if ui.add(w::secondary_button("Cancelar")).clicked() {
                outcome = DialogOutcome::Cancel;
            }
        });
    });
    outcome
}

fn error_line(ui: &mut Ui, error: &Option<String>) {
    if let Some(message) = error {
        ui.add_space(2.0);
        Frame::new()
            .fill(DANGER_SOFT)
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(10, 7))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.label(RichText::new(message).size(12.0).color(DANGER));
            });
    }
}

fn combo_client(ui: &mut Ui, selected: &mut Option<i64>, clients: &[Client]) {
    let text = selected
        .and_then(|id| clients.iter().find(|c| c.id == id))
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "Selecione um cliente".to_string());

    egui::ComboBox::from_id_salt("form_client_combo")
        .width(ui.available_width())
        .selected_text(text)
        .show_ui(ui, |ui| {
            for client in clients {
                ui.selectable_value(selected, Some(client.id), &client.name);
            }
        });
}

// ------------------------------------------------------------------- Cliente

pub struct ClientForm {
    pub id: Option<i64>,
    pub name: String,
    pub phone: String,
    pub email: String,
    pub address: String,
    pub notes: String,
    pub error: Option<String>,
}

impl ClientForm {
    pub fn new() -> Self {
        Self {
            id: None,
            name: String::new(),
            phone: String::new(),
            email: String::new(),
            address: String::new(),
            notes: String::new(),
            error: None,
        }
    }

    pub fn edit(client: &Client) -> Self {
        Self {
            id: Some(client.id),
            name: client.name.clone(),
            phone: client.phone.clone(),
            email: client.email.clone(),
            address: client.address.clone(),
            notes: client.notes.clone(),
            error: None,
        }
    }

    pub fn title(&self) -> &'static str {
        if self.id.is_some() {
            "Editar cliente"
        } else {
            "Novo cliente"
        }
    }

    pub fn show(&mut self, ui: &mut Ui) -> DialogOutcome {
        w::field(ui, "Nome completo *", |ui| {
            w::input(ui, &mut self.name, "Ex.: Maria Oliveira");
        });

        ui.columns(2, |cols| {
            w::field(&mut cols[0], "Telefone / WhatsApp", |ui| {
                w::input(ui, &mut self.phone, "(00) 00000-0000");
            });
            w::field(&mut cols[1], "E-mail", |ui| {
                w::input(ui, &mut self.email, "cliente@email.com");
            });
        });

        w::field(ui, "Endereço", |ui| {
            w::input(ui, &mut self.address, "Rua, número — cidade/UF");
        });

        w::field(ui, "Observações", |ui| {
            w::input_multiline(
                ui,
                &mut self.notes,
                "Preferências de contato, referências, etc.",
                2,
            );
        });

        error_line(ui, &self.error);
        ui.add_space(8.0);
        footer(ui)
    }

    pub fn save(&mut self, store: &Store) -> DbResult<String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err("Informe o nome do cliente.".to_string());
        }

        let message = match self.id {
            Some(id) => {
                store.update_client(
                    id,
                    name,
                    self.phone.trim(),
                    self.email.trim(),
                    self.address.trim(),
                    self.notes.trim(),
                )?;
                "Cliente atualizado com sucesso."
            }
            None => {
                store.insert_client(
                    name,
                    self.phone.trim(),
                    self.email.trim(),
                    self.address.trim(),
                    self.notes.trim(),
                )?;
                "Cliente cadastrado com sucesso."
            }
        };
        Ok(message.to_string())
    }
}

// ---------------------------------------------------------------- Equipamento

pub struct EquipmentForm {
    pub id: Option<i64>,
    pub client_id: Option<i64>,
    pub kind: String,
    pub brand: String,
    pub model: String,
    pub serial: String,
    pub notes: String,
    pub error: Option<String>,
}

impl EquipmentForm {
    pub fn new(client_id: Option<i64>) -> Self {
        Self {
            id: None,
            client_id,
            kind: String::new(),
            brand: String::new(),
            model: String::new(),
            serial: String::new(),
            notes: String::new(),
            error: None,
        }
    }

    pub fn edit(equipment: &Equipment) -> Self {
        Self {
            id: Some(equipment.id),
            client_id: Some(equipment.client_id),
            kind: equipment.kind.clone(),
            brand: equipment.brand.clone(),
            model: equipment.model.clone(),
            serial: equipment.serial.clone(),
            notes: equipment.notes.clone(),
            error: None,
        }
    }

    pub fn title(&self) -> &'static str {
        if self.id.is_some() {
            "Editar equipamento"
        } else {
            "Novo equipamento"
        }
    }

    pub fn show(&mut self, ui: &mut Ui, clients: &[Client]) -> DialogOutcome {
        w::field(ui, "Cliente *", |ui| {
            combo_client(ui, &mut self.client_id, clients);
        });

        ui.columns(2, |cols| {
            w::field(&mut cols[0], "Tipo *", |ui| {
                w::input(ui, &mut self.kind, "Geladeira, fogão, TV...");
            });
            w::field(&mut cols[1], "Marca", |ui| {
                w::input(ui, &mut self.brand, "Brastemp, LG, Electrolux...");
            });
        });

        ui.columns(2, |cols| {
            w::field(&mut cols[0], "Modelo", |ui| {
                w::input(ui, &mut self.model, "Modelo do aparelho");
            });
            w::field(&mut cols[1], "Número de série", |ui| {
                w::input(ui, &mut self.serial, "Etiqueta do fabricante");
            });
        });

        w::field(ui, "Observações", |ui| {
            w::input_multiline(
                ui,
                &mut self.notes,
                "Acessórios recebidos, avarias, senha, etc.",
                2,
            );
        });

        if clients.is_empty() {
            ui.label(
                RichText::new("Cadastre um cliente antes de adicionar equipamentos.")
                    .size(12.0)
                    .color(WARN),
            );
            ui.add_space(6.0);
        }

        error_line(ui, &self.error);
        ui.add_space(8.0);
        footer(ui)
    }

    pub fn save(&mut self, store: &Store) -> DbResult<String> {
        let client_id = self
            .client_id
            .ok_or_else(|| "Selecione o cliente responsável pelo equipamento.".to_string())?;
        let kind = self.kind.trim();
        if kind.is_empty() {
            return Err("Informe o tipo de equipamento (ex.: Geladeira).".to_string());
        }

        let message = match self.id {
            Some(id) => {
                store.update_equipment(
                    id,
                    client_id,
                    kind,
                    self.brand.trim(),
                    self.model.trim(),
                    self.serial.trim(),
                    self.notes.trim(),
                )?;
                "Equipamento atualizado com sucesso."
            }
            None => {
                store.insert_equipment(
                    client_id,
                    kind,
                    self.brand.trim(),
                    self.model.trim(),
                    self.serial.trim(),
                    self.notes.trim(),
                )?;
                "Equipamento cadastrado com sucesso."
            }
        };
        Ok(message.to_string())
    }
}

// ------------------------------------------------------------------ Orçamento

pub struct QuoteForm {
    pub id: Option<i64>,
    pub client_id: Option<i64>,
    pub equipment_id: Option<i64>,
    pub description: String,
    pub labor: String,
    pub parts: String,
    pub discount: String,
    pub status: QuoteStatus,
    pub error: Option<String>,
}

impl QuoteForm {
    pub fn new() -> Self {
        Self {
            id: None,
            client_id: None,
            equipment_id: None,
            description: String::new(),
            labor: String::new(),
            parts: String::new(),
            discount: String::new(),
            status: QuoteStatus::Pendente,
            error: None,
        }
    }

    pub fn edit(quote: &Quote) -> Self {
        Self {
            id: Some(quote.id),
            client_id: Some(quote.client_id),
            equipment_id: quote.equipment_id,
            description: quote.description.clone(),
            labor: money_plain(quote.labor_cents),
            parts: money_plain(quote.parts_cents),
            discount: money_plain(quote.discount_cents),
            status: quote.status,
            error: None,
        }
    }

    pub fn title(&self) -> &'static str {
        if self.id.is_some() {
            "Editar orçamento"
        } else {
            "Novo orçamento"
        }
    }

    fn total_preview(&self) -> Option<i64> {
        let labor = parse_money(&self.labor)?;
        let parts = parse_money(&self.parts)?;
        let discount = parse_money(&self.discount)?;
        Some((labor + parts - discount).max(0))
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        clients: &[Client],
        equipments: &[Equipment],
    ) -> DialogOutcome {
        w::field(ui, "Cliente *", |ui| {
            combo_client(ui, &mut self.client_id, clients);
        });

        // Mantém o equipamento coerente com o cliente escolhido.
        if let Some(equipment_id) = self.equipment_id {
            let belongs = equipments
                .iter()
                .any(|e| e.id == equipment_id && Some(e.client_id) == self.client_id);
            if !belongs {
                self.equipment_id = None;
            }
        }

        w::field(ui, "Equipamento", |ui| {
            let available: Vec<&Equipment> = equipments
                .iter()
                .filter(|e| self.client_id.map(|id| e.client_id == id).unwrap_or(false))
                .collect();

            let selected_text = self
                .equipment_id
                .and_then(|id| available.iter().find(|e| e.id == id))
                .map(|e| e.title())
                .unwrap_or_else(|| {
                    if available.is_empty() {
                        "Nenhum equipamento do cliente".to_string()
                    } else {
                        "Sem equipamento (serviço avulso)".to_string()
                    }
                });

            egui::ComboBox::from_id_salt("form_quote_equipment")
                .width(ui.available_width())
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.equipment_id,
                        None,
                        "Sem equipamento (serviço avulso)",
                    );
                    for equipment in &available {
                        ui.selectable_value(
                            &mut self.equipment_id,
                            Some(equipment.id),
                            equipment.title(),
                        );
                    }
                });
        });

        w::field(ui, "Descrição do serviço", |ui| {
            w::input_multiline(
                ui,
                &mut self.description,
                "Defeito relatado, diagnóstico e serviço a executar...",
                3,
            );
        });

        ui.columns(3, |cols| {
            w::field(&mut cols[0], "Mão de obra (R$)", |ui| {
                w::input(ui, &mut self.labor, "0,00");
            });
            w::field(&mut cols[1], "Peças (R$)", |ui| {
                w::input(ui, &mut self.parts, "0,00");
            });
            w::field(&mut cols[2], "Desconto (R$)", |ui| {
                w::input(ui, &mut self.discount, "0,00");
            });
        });

        let total = self.total_preview();
        ui.columns(2, |cols| {
            Frame::new()
                .fill(SOFT)
                .stroke(egui::Stroke::new(1.0, BORDER))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::symmetric(12, 10))
                .show(&mut cols[0], |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Total do orçamento").size(12.0).color(MUTED));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let text = total.map(money).unwrap_or_else(|| "—".to_string());
                            ui.label(RichText::new(text).size(18.0).strong().color(PRIMARY));
                        });
                    });
                });

            w::field(&mut cols[1], "Situação", |ui| {
                egui::ComboBox::from_id_salt("form_quote_status")
                    .width(ui.available_width())
                    .selected_text(self.status.label())
                    .show_ui(ui, |ui| {
                        for status in QuoteStatus::ALL {
                            ui.selectable_value(&mut self.status, status, status.label());
                        }
                    });
            });
        });

        error_line(ui, &self.error);
        ui.add_space(8.0);
        footer(ui)
    }

    pub fn save(&mut self, store: &Store) -> DbResult<String> {
        let client_id = self
            .client_id
            .ok_or_else(|| "Selecione o cliente do orçamento.".to_string())?;

        let labor = parse_money(&self.labor).ok_or_else(|| {
            "Valor de mão de obra inválido. Use, por exemplo, 120,50.".to_string()
        })?;
        let parts = parse_money(&self.parts)
            .ok_or_else(|| "Valor de peças inválido. Use, por exemplo, 89,90.".to_string())?;
        let discount = parse_money(&self.discount)
            .ok_or_else(|| "Desconto inválido. Use, por exemplo, 10,00.".to_string())?;

        if labor < 0 || parts < 0 || discount < 0 {
            return Err("Os valores não podem ser negativos.".to_string());
        }
        if discount > labor + parts {
            return Err(format!(
                "O desconto ({}) é maior que o total de peças e mão de obra ({}).",
                money(discount),
                money(labor + parts)
            ));
        }

        let message = match self.id {
            Some(id) => {
                store.update_quote(
                    id,
                    client_id,
                    self.equipment_id,
                    self.description.trim(),
                    self.status,
                    labor,
                    parts,
                    discount,
                )?;
                "Orçamento atualizado com sucesso."
            }
            None => {
                store.insert_quote(
                    client_id,
                    self.equipment_id,
                    self.description.trim(),
                    self.status,
                    labor,
                    parts,
                    discount,
                )?;
                "Orçamento criado com sucesso."
            }
        };
        Ok(message.to_string())
    }
}

// ------------------------------------------------------------------ Pagamento

pub struct PaymentForm {
    pub id: Option<i64>,
    pub quote_id: Option<i64>,
    pub amount: String,
    pub method: String,
    pub paid_on: String,
    pub notes: String,
    pub error: Option<String>,
}

impl PaymentForm {
    pub fn new() -> Self {
        Self {
            id: None,
            quote_id: None,
            amount: String::new(),
            method: "Pix".to_string(),
            paid_on: today_br(),
            notes: String::new(),
            error: None,
        }
    }

    pub fn edit(payment: &Payment) -> Self {
        Self {
            id: Some(payment.id),
            quote_id: Some(payment.quote_id),
            amount: money_plain(payment.amount_cents),
            method: payment.method.clone(),
            paid_on: iso_to_br(&payment.paid_on),
            notes: payment.notes.clone(),
            error: None,
        }
    }

    pub fn title(&self) -> &'static str {
        if self.id.is_some() {
            "Editar pagamento"
        } else {
            "Novo pagamento"
        }
    }

    pub fn show(&mut self, ui: &mut Ui, quotes: &[Quote]) -> DialogOutcome {
        let selected = self
            .quote_id
            .and_then(|id| quotes.iter().find(|q| q.id == id));

        w::field(ui, "Orçamento *", |ui| {
            let selected_text = selected
                .map(|q| format!("#{} · {}", q.id, q.client_name))
                .unwrap_or_else(|| "Selecione um orçamento".to_string());

            egui::ComboBox::from_id_salt("form_payment_quote")
                .width(ui.available_width())
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for quote in quotes {
                        let label = format!(
                            "#{} · {} — {}",
                            quote.id, quote.client_name, quote.equipment_label
                        );
                        ui.selectable_value(&mut self.quote_id, Some(quote.id), label);
                    }
                });
        });

        if let Some(quote) = selected {
            Frame::new()
                .fill(SOFT)
                .stroke(egui::Stroke::new(1.0, BORDER))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::symmetric(12, 9))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "Total {}  ·  Pago {}",
                                money(quote.total_cents()),
                                money(quote.paid_cents)
                            ))
                            .size(12.0)
                            .color(MUTED),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let balance = quote.balance_cents();
                            if balance == 0 {
                                w::badge(ui, "Quitado", SUCCESS, SUCCESS_SOFT);
                            } else {
                                ui.label(
                                    RichText::new(format!("Saldo {}", money(balance)))
                                        .size(12.5)
                                        .strong()
                                        .color(WARN),
                                );
                            }
                        });
                    });
                    ui.add_space(4.0);
                    if quote.balance_cents() > 0 {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Usar saldo restante")
                                        .size(11.5)
                                        .color(PRIMARY),
                                )
                                .small()
                                .frame(false),
                            )
                            .clicked()
                        {
                            self.amount = money_plain(quote.balance_cents());
                        }
                    }
                });
            ui.add_space(10.0);
        }

        ui.columns(2, |cols| {
            w::field(&mut cols[0], "Valor (R$) *", |ui| {
                w::input(ui, &mut self.amount, "0,00");
            });
            w::field(&mut cols[1], "Data do pagamento", |ui| {
                w::input(ui, &mut self.paid_on, "dd/mm/aaaa");
            });
        });

        w::field(ui, "Forma de pagamento", |ui| {
            egui::ComboBox::from_id_salt("form_payment_method")
                .width(220.0)
                .selected_text(&self.method)
                .show_ui(ui, |ui| {
                    for method in PAYMENT_METHODS {
                        ui.selectable_value(&mut self.method, method.to_string(), method);
                    }
                });
        });

        w::field(ui, "Observações", |ui| {
            w::input_multiline(ui, &mut self.notes, "Comprovante, parcelas, etc.", 2);
        });

        error_line(ui, &self.error);
        ui.add_space(8.0);
        footer(ui)
    }

    pub fn save(&mut self, store: &Store) -> DbResult<String> {
        let quote_id = self
            .quote_id
            .ok_or_else(|| "Selecione o orçamento que recebeu o pagamento.".to_string())?;

        let amount = parse_money(&self.amount)
            .ok_or_else(|| "Valor inválido. Use, por exemplo, 150,00.".to_string())?;
        if amount <= 0 {
            return Err("Informe um valor maior que zero.".to_string());
        }

        let paid_on = br_to_iso(&self.paid_on).ok_or_else(|| {
            "Data inválida. Use o formato dd/mm/aaaa (ex.: 13/11/2026).".to_string()
        })?;

        let message = match self.id {
            Some(id) => {
                store.update_payment(
                    id,
                    quote_id,
                    amount,
                    &self.method,
                    &paid_on,
                    self.notes.trim(),
                )?;
                "Pagamento atualizado com sucesso."
            }
            None => {
                store.insert_payment(
                    quote_id,
                    amount,
                    &self.method,
                    &paid_on,
                    self.notes.trim(),
                )?;
                "Pagamento registrado com sucesso."
            }
        };
        Ok(message.to_string())
    }
}

#[cfg(test)]
mod form_fit_tests {
    use super::*;

    fn height(width: f32, add: impl FnOnce(&mut Ui)) -> f32 {
        let ctx = egui::Context::default();
        crate::theme::apply(&ctx);
        let mut input = egui::RawInput::default();
        input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1220.0, 790.0),
        ));

        let mut measured = 0.0;
        let mut add = Some(add);
        let mut output = ctx.run_ui(input, |ui| {
            ui.set_width(width);
            if let Some(add) = add.take() {
                add(ui);
            }
            measured = ui.min_rect().height();
        });
        output.textures_delta.clear();
        measured
    }

    /// Garante que os formulários caibam inteiros em uma janela de 640 px de
    /// altura, sem acionar a barra de rolagem dos modais.
    #[test]
    fn forms_fit_without_scrolling() {
        let heights = [
            (
                "cliente",
                height(496.0, |ui| {
                    let _ = ClientForm::new().show(ui);
                }),
            ),
            (
                "equipamento",
                height(556.0, |ui| {
                    let _ = EquipmentForm::new(None).show(ui, &[]);
                }),
            ),
            (
                "orçamento",
                height(616.0, |ui| {
                    let _ = QuoteForm::new().show(ui, &[], &[]);
                }),
            ),
            (
                "pagamento",
                height(556.0, |ui| {
                    let _ = PaymentForm::new().show(ui, &[]);
                }),
            ),
        ];

        // 640 px de janela − 44 da moldura − 52 do título − 24 de respiro.
        let budget = 520.0;
        for (name, value) in heights {
            assert!(
                value <= budget,
                "formulário de {name} mede {value:.1} px (limite {budget:.0} px)"
            );
        }
    }
}
