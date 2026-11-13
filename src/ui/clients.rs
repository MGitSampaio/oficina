//! Página de clientes.

use egui::{Align, Grid, Layout, RichText, Ui};

use crate::app::{App, DeleteTarget, Page};
use crate::theme::*;
use crate::ui::widgets as w;
use crate::util::iso_to_br;

enum ClientAction {
    Edit,
    Equipment,
    Delete,
}

impl App {
    pub(crate) fn page_clients(&mut self, ui: &mut Ui) {
        let mut new_clicked = false;
        let mut action: Option<(i64, ClientAction)> = None;

        ui.horizontal(|ui| {
            w::search_input(
                ui,
                &mut self.search,
                "Buscar por nome, telefone, e-mail ou endereço...",
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(w::primary_button("+ Novo cliente")).clicked() {
                    new_clicked = true;
                }
            });
        });
        ui.add_space(12.0);

        let needle = self.search.trim().to_lowercase();
        let rows: Vec<_> = self
            .clients
            .iter()
            .filter(|c| {
                needle.is_empty()
                    || c.name.to_lowercase().contains(&needle)
                    || c.phone.to_lowercase().contains(&needle)
                    || c.email.to_lowercase().contains(&needle)
                    || c.address.to_lowercase().contains(&needle)
            })
            .collect();
        let filtered = rows.len();
        let total = self.clients.len();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                w::card(ui, |ui| {
                    if rows.is_empty() {
                        if total == 0 {
                            w::empty_state(
                                ui,
                                "Nenhum cliente cadastrado",
                                "Clique em “Novo cliente” para começar o cadastro.",
                            );
                        } else {
                            w::empty_state(
                                ui,
                                "Nenhum cliente encontrado",
                                "Tente outro termo de busca ou limpe o filtro.",
                            );
                        }
                    } else {
                        w::table_scroll(ui, "clients_scroll", |ui| {
                            Grid::new("clients_grid")
                                .num_columns(5)
                                .spacing(egui::vec2(18.0, 12.0))
                                .striped(true)
                                .show(ui, |ui| {
                                    w::th(ui, "Nome");
                                    w::th(ui, "Contato");
                                    w::th(ui, "Endereço");
                                    w::th(ui, "Equip.");
                                    w::th(ui, "Ações");
                                    ui.end_row();

                                    for client in &rows {
                                        ui.vertical(|ui| {
                                            let name = ui.label(
                                                RichText::new(&client.name)
                                                    .size(13.0)
                                                    .strong()
                                                    .color(TEXT),
                                            );
                                            if !client.notes.is_empty() {
                                                name.on_hover_text(&client.notes);
                                            }
                                            ui.label(
                                                RichText::new(format!(
                                                    "Cliente desde {}",
                                                    iso_to_br(&client.created_at)
                                                ))
                                                .size(10.0)
                                                .color(FAINT),
                                            );
                                        });

                                        ui.vertical(|ui| {
                                            if client.phone.is_empty() && client.email.is_empty() {
                                                ui.label(
                                                    RichText::new("Sem contato")
                                                        .size(11.5)
                                                        .color(FAINT),
                                                );
                                            } else {
                                                if !client.phone.is_empty() {
                                                    ui.label(
                                                        RichText::new(&client.phone).size(12.0),
                                                    );
                                                }
                                                if !client.email.is_empty() {
                                                    ui.label(
                                                        RichText::new(w::ellipsis(
                                                            &client.email,
                                                            30,
                                                        ))
                                                        .size(11.0)
                                                        .color(MUTED),
                                                    );
                                                }
                                            }
                                        });

                                        ui.label(
                                            RichText::new(w::ellipsis(&client.address, 38))
                                                .size(11.5)
                                                .color(MUTED),
                                        )
                                        .on_hover_text(
                                            if client.address.is_empty() {
                                                "Sem endereço cadastrado".to_string()
                                            } else {
                                                client.address.clone()
                                            },
                                        );

                                        ui.label(
                                            RichText::new(client.equipment_count.to_string())
                                                .size(12.0)
                                                .color(MUTED),
                                        );

                                        ui.allocate_ui_with_layout(
                                            egui::vec2(188.0, 28.0),
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                if w::small_danger(ui, "Excluir").clicked() {
                                                    action =
                                                        Some((client.id, ClientAction::Delete));
                                                }
                                                if w::small_action(ui, "Equip.")
                                                    .on_hover_text("Ver equipamentos deste cliente")
                                                    .clicked()
                                                {
                                                    action =
                                                        Some((client.id, ClientAction::Equipment));
                                                }
                                                if w::small_action(ui, "Editar").clicked() {
                                                    action = Some((client.id, ClientAction::Edit));
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
                    RichText::new(format!("{filtered} de {total} cliente(s)"))
                        .size(11.0)
                        .color(FAINT),
                );
            });

        if new_clicked {
            self.open_client_form(None);
        }

        if let Some((id, action)) = action {
            match action {
                ClientAction::Edit => self.open_client_form(Some(id)),
                ClientAction::Equipment => {
                    self.equipment_client_filter = Some(id);
                    self.search.clear();
                    self.page = Page::Equipment;
                }
                ClientAction::Delete => self.ask_delete(DeleteTarget::Client(id)),
            }
        }
    }
}
