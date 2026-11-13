//! Página de pagamentos recebidos.

use egui::{Align, Grid, Layout, RichText, Ui};

use crate::app::{App, DeleteTarget};
use crate::theme::*;
use crate::ui::widgets as w;
use crate::util::{iso_to_br, money};

enum PaymentAction {
    Edit,
    Delete,
}

impl App {
    pub(crate) fn page_payments(&mut self, ui: &mut Ui) {
        let mut new_clicked = false;
        let mut action: Option<(i64, PaymentAction)> = None;

        ui.horizontal(|ui| {
            w::search_input(
                ui,
                &mut self.search,
                "Buscar por cliente, orçamento ou forma de pagamento...",
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(w::primary_button("+ Novo pagamento")).clicked() {
                    new_clicked = true;
                }
            });
        });
        ui.add_space(12.0);

        let needle = self.search.trim().to_lowercase();
        let rows: Vec<_> = self
            .payments
            .iter()
            .filter(|p| {
                needle.is_empty()
                    || p.client_name.to_lowercase().contains(&needle)
                    || p.quote_description.to_lowercase().contains(&needle)
                    || p.method.to_lowercase().contains(&needle)
                    || iso_to_br(&p.paid_on).contains(&needle)
            })
            .collect();
        let filtered = rows.len();
        let total = self.payments.len();
        let sum: i64 = rows.iter().map(|p| p.amount_cents).sum();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                w::card(ui, |ui| {
                    if rows.is_empty() {
                        if total == 0 {
                            w::empty_state(
                                ui,
                                "Nenhum pagamento registrado",
                                "Registre entradas pelo botão “Novo pagamento” ou pelo botão Pagar de um orçamento.",
                            );
                        } else {
                            w::empty_state(
                                ui,
                                "Nenhum pagamento encontrado",
                                "Tente outro termo de busca.",
                            );
                        }
                    } else {
                        w::table_scroll(ui, "payments_scroll", |ui| {
                            Grid::new("payments_grid")
                                .num_columns(5)
                                .spacing(egui::vec2(18.0, 12.0))
                                .striped(true)
                                .show(ui, |ui| {
                                    w::th(ui, "Data");
                                    w::th(ui, "Cliente / Orçamento");
                                    w::th(ui, "Forma");
                                    w::th(ui, "Valor");
                                    w::th(ui, "Ações");
                                    ui.end_row();

                                    for payment in &rows {
                                        ui.label(
                                            RichText::new(iso_to_br(&payment.paid_on))
                                                .size(12.0)
                                                .color(MUTED),
                                        );

                                        ui.vertical(|ui| {
                                            let name = ui.label(
                                                RichText::new(&payment.client_name)
                                                    .size(13.0)
                                                    .strong()
                                                    .color(TEXT),
                                            );
                                            if !payment.notes.is_empty() {
                                                name.on_hover_text(&payment.notes);
                                            }
                                            ui.label(
                                                RichText::new(format!(
                                                    "#{} · {}",
                                                    payment.quote_id,
                                                    w::ellipsis(&payment.quote_description, 38)
                                                ))
                                                .size(10.5)
                                                .color(MUTED),
                                            )
                                            .on_hover_text(format!(
                                                "Total do orçamento: {}",
                                                money(payment.quote_total_cents)
                                            ));
                                        });

                                        w::badge(ui, &payment.method, INFO, INFO_SOFT);

                                        ui.label(
                                            RichText::new(money(payment.amount_cents))
                                                .size(13.0)
                                                .strong()
                                                .color(SUCCESS),
                                        );

                                        ui.allocate_ui_with_layout(
                                            egui::vec2(150.0, 28.0),
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                if w::small_danger(ui, "Excluir").clicked() {
                                                    action = Some((payment.id, PaymentAction::Delete));
                                                }
                                                if w::small_action(ui, "Editar").clicked() {
                                                    action = Some((payment.id, PaymentAction::Edit));
                                                }
                                            },
                                        );

                                        ui.end_row();
                                    }
                            });
                        });
                    }
                });

                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!(
                        "{filtered} de {total} pagamento(s) · total {}",
                        money(sum)
                    ))
                    .size(11.0)
                    .color(FAINT),
                );
            });

        if new_clicked {
            if self.quotes.is_empty() {
                self.notify_error("Crie um orçamento antes de registrar pagamentos.");
            } else {
                self.open_payment_form(None);
            }
        }

        if let Some((id, action)) = action {
            match action {
                PaymentAction::Edit => self.open_payment_form(Some(id)),
                PaymentAction::Delete => self.ask_delete(DeleteTarget::Payment(id)),
            }
        }
    }
}
