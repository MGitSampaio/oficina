//! Página de orçamentos.

use egui::{Align, Grid, Layout, RichText, Ui};

use crate::app::{App, DeleteTarget};
use crate::model::QuoteStatus;
use crate::theme::*;
use crate::ui::widgets as w;
use crate::util::{iso_to_br, money};

enum QuoteAction {
    Edit,
    Pay,
    Print,
    Delete,
}

impl App {
    pub(crate) fn page_quotes(&mut self, ui: &mut Ui) {
        let mut new_clicked = false;
        let mut action: Option<(i64, QuoteAction)> = None;

        ui.horizontal(|ui| {
            w::search_input(
                ui,
                &mut self.search,
                "Buscar por cliente, equipamento ou descrição...",
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(w::primary_button("+ Novo orçamento")).clicked() {
                    new_clicked = true;
                }
            });
        });

        ui.add_space(8.0);

        let mut filters: Vec<(Option<QuoteStatus>, String, usize)> = Vec::new();
        filters.push((None, "Todos".to_string(), self.quotes.len()));
        for status in QuoteStatus::ALL {
            let count = self.quotes.iter().filter(|q| q.status == status).count();
            filters.push((Some(status), status.label().to_string(), count));
        }

        ui.horizontal_wrapped(|ui| {
            for (filter, label, count) in &filters {
                let text = format!("{label}  {count}");
                let selected = self.status_filter == *filter;
                let response = ui.selectable_label(
                    selected,
                    RichText::new(text)
                        .size(12.0)
                        .color(if selected { PRIMARY } else { MUTED }),
                );
                if response.clicked() {
                    self.status_filter = *filter;
                }
            }
        });

        ui.add_space(10.0);

        let needle = self.search.trim().to_lowercase();
        let status_filter = self.status_filter;
        let rows: Vec<_> = self
            .quotes
            .iter()
            .filter(|q| status_filter.map(|s| q.status == s).unwrap_or(true))
            .filter(|q| {
                needle.is_empty()
                    || q.client_name.to_lowercase().contains(&needle)
                    || q.equipment_label.to_lowercase().contains(&needle)
                    || q.description.to_lowercase().contains(&needle)
                    || format!("#{}", q.id).contains(&needle)
            })
            .collect();
        let filtered = rows.len();
        let total = self.quotes.len();
        let sum_total: i64 = rows.iter().map(|q| q.total_cents()).sum();
        let sum_paid: i64 = rows.iter().map(|q| q.paid_cents).sum();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                w::card(ui, |ui| {
                    if rows.is_empty() {
                        if total == 0 {
                            w::empty_state(
                                ui,
                                "Nenhum orçamento cadastrado",
                                "Crie um orçamento para registrar serviços, peças e valores.",
                            );
                        } else {
                            w::empty_state(
                                ui,
                                "Nenhum orçamento encontrado",
                                "Ajuste a busca ou selecione outra situação.",
                            );
                        }
                    } else {
                        w::table_scroll(ui, "quotes_scroll", |ui| {
                            Grid::new("quotes_grid")
                                .num_columns(6)
                                .spacing(egui::vec2(16.0, 12.0))
                                .striped(true)
                                .show(ui, |ui| {
                                    w::th(ui, "Nº");
                                    w::th(ui, "Cliente / Equipamento");
                                    w::th(ui, "Situação");
                                    w::th(ui, "Total");
                                    w::th(ui, "Saldo");
                                    w::th(ui, "Ações");
                                    ui.end_row();

                                    for quote in &rows {
                                        ui.label(
                                            RichText::new(format!("#{}", quote.id))
                                                .size(11.5)
                                                .color(FAINT),
                                        );

                                        ui.vertical(|ui| {
                                            let name = ui.label(
                                                RichText::new(&quote.client_name)
                                                    .size(13.0)
                                                    .strong()
                                                    .color(TEXT),
                                            );
                                            if !quote.description.is_empty() {
                                                name.on_hover_text(&quote.description);
                                            }
                                            ui.label(
                                                RichText::new(w::ellipsis(
                                                    &quote.equipment_label,
                                                    30,
                                                ))
                                                .size(10.5)
                                                .color(MUTED),
                                            );
                                            if !quote.description.is_empty() {
                                                ui.label(
                                                    RichText::new(w::ellipsis(
                                                        &quote.description,
                                                        36,
                                                    ))
                                                    .size(10.0)
                                                    .color(FAINT),
                                                );
                                            }
                                            ui.label(
                                                RichText::new(format!(
                                                    "Aberto em {}",
                                                    iso_to_br(&quote.created_at)
                                                ))
                                                .size(10.0)
                                                .color(FAINT),
                                            );
                                        });

                                        w::status_badge(ui, quote.status);

                                        ui.vertical(|ui| {
                                            ui.label(
                                                RichText::new(money(quote.total_cents()))
                                                    .size(12.5)
                                                    .strong()
                                                    .color(TEXT),
                                            );
                                            if quote.paid_cents > 0 {
                                                ui.label(
                                                    RichText::new(format!(
                                                        "pago {}",
                                                        money(quote.paid_cents)
                                                    ))
                                                    .size(10.0)
                                                    .color(SUCCESS),
                                                );
                                            }
                                        });

                                        if quote.is_paid() || quote.balance_cents() == 0 {
                                            ui.label(
                                                RichText::new("Quitado")
                                                    .size(11.5)
                                                    .strong()
                                                    .color(SUCCESS),
                                            );
                                        } else {
                                            ui.label(
                                                RichText::new(money(quote.balance_cents()))
                                                    .size(12.0)
                                                    .color(WARN),
                                            );
                                        }

                                        ui.allocate_ui_with_layout(
                                            egui::vec2(268.0, 28.0),
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                if w::small_danger(ui, "Excluir").clicked() {
                                                    action = Some((quote.id, QuoteAction::Delete));
                                                }
                                                if quote.balance_cents() > 0
                                                    && w::small_success(ui, "Pagar").clicked()
                                                {
                                                    action = Some((quote.id, QuoteAction::Pay));
                                                }
                                                if w::small_action(ui, "Imprimir")
                                                    .on_hover_text("Abrir orçamento para impressão")
                                                    .clicked()
                                                {
                                                    action = Some((quote.id, QuoteAction::Print));
                                                }
                                                if w::small_action(ui, "Editar").clicked() {
                                                    action = Some((quote.id, QuoteAction::Edit));
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
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "{filtered} de {total} orçamento(s) · total {} · recebido {}",
                            money(sum_total),
                            money(sum_paid)
                        ))
                        .size(11.0)
                        .color(FAINT),
                    );
                });
            });

        if new_clicked {
            if self.clients.is_empty() {
                self.notify_error("Cadastre um cliente antes de criar um orçamento.");
            } else {
                self.open_quote_form(None);
            }
        }

        if let Some((id, action)) = action {
            match action {
                QuoteAction::Edit => self.open_quote_form(Some(id)),
                QuoteAction::Pay => self.open_payment_for_quote(id),
                QuoteAction::Print => self.print_quote(id),
                QuoteAction::Delete => self.ask_delete(DeleteTarget::Quote(id)),
            }
        }
    }
}
