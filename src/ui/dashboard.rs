//! Painel inicial com indicadores e atalhos.

use egui::{Align, Color32, Layout, RichText, Ui};

use crate::app::App;
use crate::theme::*;
use crate::ui::widgets as w;
use crate::util::money;

const PURPLE: Color32 = Color32::from_rgb(0x7C, 0x3A, 0xED);

impl App {
    pub(crate) fn page_dashboard(&mut self, ui: &mut Ui) {
        let mut new_quote = false;
        let mut new_client = false;
        let mut new_payment = false;
        let mut seed = false;
        let mut edit_quote: Option<i64> = None;
        let mut pay_quote: Option<i64> = None;
        let mut go_clients = false;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.clients.is_empty() {
                    w::card(ui, |ui| {
                        ui.set_min_height(340.0);
                        ui.vertical_centered(|ui| {
                            ui.add_space(50.0);
                            ui.label(
                                RichText::new("Bem-vindo à Oficina")
                                    .size(26.0)
                                    .strong()
                                    .color(TEXT),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(
                                    "Controle clientes, equipamentos, orçamentos e pagamentos em um só lugar.",
                                )
                                .size(13.5)
                                .color(MUTED),
                            );
                            ui.add_space(24.0);
                            if ui
                                .add(w::primary_button("Cadastrar o primeiro cliente"))
                                .clicked()
                            {
                                new_client = true;
                            }
                            ui.add_space(10.0);
                            if ui
                                .add(w::secondary_button("Carregar dados de exemplo"))
                                .clicked()
                            {
                                seed = true;
                            }
                            ui.add_space(26.0);
                            ui.label(
                                RichText::new(
                                    "Os dados ficam salvos localmente, no arquivo oficina.db.",
                                )
                                .size(11.5)
                                .color(FAINT),
                            );
                        });
                    });
                    return;
                }

                ui.columns(4, |cols| {
                    w::stat_card(
                        &mut cols[0],
                        "Clientes",
                        &self.stats.clients.to_string(),
                        "cadastrados na oficina",
                        PRIMARY,
                    );
                    w::stat_card(
                        &mut cols[1],
                        "Equipamentos",
                        &self.stats.equipment.to_string(),
                        "recebidos para conserto",
                        PURPLE,
                    );
                    w::stat_card(
                        &mut cols[2],
                        "A receber",
                        &money(self.stats.receivable_cents),
                        &format!(
                            "{} orçamento(s) com saldo em aberto",
                            self.stats.payable_quotes
                        ),
                        WARN,
                    );
                    w::stat_card(
                        &mut cols[3],
                        "Recebido no mês",
                        &money(self.stats.month_revenue_cents),
                        &format!("Total histórico: {}", money(self.stats.total_revenue_cents)),
                        SUCCESS,
                    );
                });

                ui.add_space(14.0);

                ui.horizontal(|ui| {
                    if ui.add(w::primary_button("+ Novo orçamento")).clicked() {
                        new_quote = true;
                    }
                    ui.add_space(2.0);
                    if ui.add(w::secondary_button("+ Novo cliente")).clicked() {
                        new_client = true;
                    }
                    ui.add_space(2.0);
                    if ui.add(w::secondary_button("Registrar pagamento")).clicked() {
                        new_payment = true;
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} orçamento(s) em aberto no total",
                                self.stats.open_quotes
                            ))
                            .size(12.0)
                            .color(MUTED),
                        );
                    });
                });

                ui.add_space(14.0);

                ui.columns(2, |cols| {
                    // ------------------------------------------ Orçamentos recentes
                    w::card(&mut cols[0], |ui| {
                        ui.horizontal(|ui| {
                            w::section_title(ui, "Orçamentos recentes");
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(
                                    RichText::new(format!("{} no total", self.quotes.len()))
                                        .size(11.5)
                                        .color(FAINT),
                                );
                            });
                        });
                        ui.add_space(6.0);
                        w::divider(ui);
                        ui.add_space(2.0);

                        if self.quotes.is_empty() {
                            w::empty_state(
                                ui,
                                "Nenhum orçamento ainda",
                                "Crie um orçamento para acompanhar o serviço.",
                            );
                        }

                        for quote in self.quotes.iter().take(6) {
                            ui.add_space(7.0);
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(&quote.client_name).size(13.0).strong(),
                                    );
                                    ui.label(
                                        RichText::new(format!(
                                            "#{} · {}",
                                            quote.id, quote.equipment_label
                                        ))
                                        .size(11.0)
                                        .color(MUTED),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if w::small_action(ui, "Abrir").clicked() {
                                        edit_quote = Some(quote.id);
                                    }
                                    ui.add_space(2.0);
                                    w::status_badge(ui, quote.status);
                                    ui.add_space(2.0);
                                    ui.label(
                                        RichText::new(money(quote.total_cents()))
                                            .size(12.5)
                                            .strong()
                                            .color(TEXT),
                                    );
                                });
                            });
                            ui.add_space(3.0);
                            w::divider(ui);
                        }
                    });

                    // -------------------------------------------------- A receber
                    w::card(&mut cols[1], |ui| {
                        ui.horizontal(|ui| {
                            w::section_title(ui, "Serviços a receber");
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(
                                    RichText::new(money(self.stats.receivable_cents))
                                        .size(12.5)
                                        .strong()
                                        .color(WARN),
                                );
                            });
                        });
                        ui.add_space(6.0);
                        w::divider(ui);
                        ui.add_space(2.0);

                        let mut pending: Vec<_> = self
                            .quotes
                            .iter()
                            .filter(|q| {
                                q.status != crate::model::QuoteStatus::Recusado
                                    && q.balance_cents() > 0
                            })
                            .collect();
                        pending.sort_by(|a, b| b.balance_cents().cmp(&a.balance_cents()));

                        if pending.is_empty() {
                            ui.add_space(18.0);
                            ui.vertical_centered(|ui| {
                                w::badge(ui, "Tudo em dia", SUCCESS, SUCCESS_SOFT);
                                ui.add_space(6.0);
                                ui.label(
                                    RichText::new("Nenhum valor em aberto.")
                                        .size(12.0)
                                        .color(MUTED),
                                );
                            });
                            ui.add_space(18.0);
                        }

                        for quote in pending.iter().take(5) {
                            ui.add_space(7.0);
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(&quote.client_name).size(13.0).strong(),
                                    );
                                    ui.label(
                                        RichText::new(format!(
                                            "#{} · {}",
                                            quote.id, quote.equipment_label
                                        ))
                                        .size(11.0)
                                        .color(MUTED),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if w::small_success(ui, "Receber").clicked() {
                                        pay_quote = Some(quote.id);
                                    }
                                    ui.add_space(2.0);
                                    ui.label(
                                        RichText::new(money(quote.balance_cents()))
                                            .size(12.5)
                                            .strong()
                                            .color(WARN),
                                    );
                                });
                            });
                            ui.add_space(3.0);
                            w::divider(ui);
                        }

                        ui.add_space(6.0);
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Ver todos os pagamentos")
                                        .size(11.5)
                                        .color(PRIMARY),
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            new_payment = true;
                        }
                    });
                });

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("Gerenciar clientes").size(11.5).color(MUTED),
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        go_clients = true;
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new("Dados salvos automaticamente")
                                .size(10.5)
                                .color(FAINT),
                        );
                    });
                });
            });

        if new_client {
            self.open_client_form(None);
        }
        if new_quote {
            self.open_quote_form(None);
        }
        if new_payment {
            self.open_payment_form(None);
        }
        if let Some(id) = edit_quote {
            self.open_quote_form(Some(id));
        }
        if let Some(id) = pay_quote {
            self.open_payment_for_quote(id);
        }
        if seed {
            match self.store.seed_demo() {
                Ok(()) => {
                    self.reload();
                    self.notify_success("Dados de exemplo carregados.");
                }
                Err(err) => self.notify_error(format!("Erro ao carregar exemplos: {err}")),
            }
        }
        if go_clients {
            self.page = crate::app::Page::Clients;
        }
    }
}
