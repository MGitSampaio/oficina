//! Página de equipamentos.

use egui::{Align, Grid, Layout, RichText, Ui};

use crate::app::{App, DeleteTarget};
use crate::theme::*;
use crate::ui::widgets as w;
use crate::util::iso_to_br;

enum EquipmentAction {
    Edit,
    NewQuote,
    Delete,
}

impl App {
    pub(crate) fn page_equipment(&mut self, ui: &mut Ui) {
        let mut new_clicked = false;
        let mut action: Option<(i64, EquipmentAction)> = None;

        let filter_label = self
            .equipment_client_filter
            .and_then(|id| self.clients.iter().find(|c| c.id == id))
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Todos os clientes".to_string());

        ui.horizontal(|ui| {
            w::search_input(
                ui,
                &mut self.search,
                "Buscar por tipo, marca, modelo, série ou cliente...",
            );
            ui.add_space(4.0);
            let clients = &self.clients;
            egui::ComboBox::from_id_salt("equipment_client_filter")
                .width(200.0)
                .selected_text(filter_label)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.equipment_client_filter,
                        None,
                        "Todos os clientes",
                    );
                    for client in clients {
                        ui.selectable_value(
                            &mut self.equipment_client_filter,
                            Some(client.id),
                            &client.name,
                        );
                    }
                });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(w::primary_button("+ Novo equipamento")).clicked() {
                    new_clicked = true;
                }
            });
        });
        ui.add_space(12.0);

        let needle = self.search.trim().to_lowercase();
        let client_filter = self.equipment_client_filter;
        let rows: Vec<_> = self
            .equipments
            .iter()
            .filter(|e| client_filter.map(|id| e.client_id == id).unwrap_or(true))
            .filter(|e| {
                needle.is_empty()
                    || e.kind.to_lowercase().contains(&needle)
                    || e.brand.to_lowercase().contains(&needle)
                    || e.model.to_lowercase().contains(&needle)
                    || e.serial.to_lowercase().contains(&needle)
                    || e.client_name.to_lowercase().contains(&needle)
            })
            .collect();
        let filtered = rows.len();
        let total = self.equipments.len();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                w::card(ui, |ui| {
                    if rows.is_empty() {
                        if total == 0 {
                            w::empty_state(
                                ui,
                                "Nenhum equipamento cadastrado",
                                "Receba um aparelho clicando em “Novo equipamento”.",
                            );
                        } else {
                            w::empty_state(
                                ui,
                                "Nenhum equipamento encontrado",
                                "Ajuste a busca ou o filtro de cliente.",
                            );
                        }
                    } else {
                        w::table_scroll(ui, "equipment_scroll", |ui| {
                            Grid::new("equipment_grid")
                                .num_columns(5)
                                .spacing(egui::vec2(18.0, 12.0))
                                .striped(true)
                                .show(ui, |ui| {
                                    w::th(ui, "Equipamento");
                                    w::th(ui, "Cliente");
                                    w::th(ui, "Orçamentos");
                                    w::th(ui, "Cadastro");
                                    w::th(ui, "Ações");
                                    ui.end_row();

                                    for item in &rows {
                                        ui.vertical(|ui| {
                                            let title = ui.label(
                                                RichText::new(item.title())
                                                    .size(13.0)
                                                    .strong()
                                                    .color(TEXT),
                                            );
                                            if !item.notes.is_empty() {
                                                title.on_hover_text(&item.notes);
                                            }
                                            ui.label(
                                                RichText::new(if item.serial.is_empty() {
                                                    "Série não informada".to_string()
                                                } else {
                                                    format!("Série {}", item.serial)
                                                })
                                                .size(10.5)
                                                .color(FAINT),
                                            );
                                        });

                                        ui.label(
                                            RichText::new(&item.client_name)
                                                .size(12.0)
                                                .color(MUTED),
                                        );

                                        ui.label(
                                            RichText::new(item.quote_count.to_string())
                                                .size(12.0)
                                                .color(MUTED),
                                        );

                                        ui.label(
                                            RichText::new(iso_to_br(&item.created_at))
                                                .size(11.5)
                                                .color(MUTED),
                                        );

                                        ui.allocate_ui_with_layout(
                                            egui::vec2(222.0, 28.0),
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                if w::small_danger(ui, "Excluir").clicked() {
                                                    action =
                                                        Some((item.id, EquipmentAction::Delete));
                                                }
                                                if w::small_action(ui, "Orçamento").clicked() {
                                                    action =
                                                        Some((item.id, EquipmentAction::NewQuote));
                                                }
                                                if w::small_action(ui, "Editar").clicked() {
                                                    action = Some((item.id, EquipmentAction::Edit));
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
                    RichText::new(format!("{filtered} de {total} equipamento(s)"))
                        .size(11.0)
                        .color(FAINT),
                );
            });

        if new_clicked {
            self.open_equipment_form(None);
        }

        if let Some((id, action)) = action {
            match action {
                EquipmentAction::Edit => self.open_equipment_form(Some(id)),
                EquipmentAction::NewQuote => {
                    let mut form = crate::ui::forms::QuoteForm::new();
                    if let Some(equipment) = self.equipments.iter().find(|e| e.id == id) {
                        form.client_id = Some(equipment.client_id);
                        form.equipment_id = Some(equipment.id);
                    }
                    self.dialog = crate::app::Dialog::Quote(form);
                }
                EquipmentAction::Delete => self.ask_delete(DeleteTarget::Equipment(id)),
            }
        }
    }
}
