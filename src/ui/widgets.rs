//! Componentes visuais reutilizáveis.

use egui::{
    Align, Align2, Button, Color32, CornerRadius, CursorIcon, FontId, Frame, Layout, Margin,
    Response, RichText, Sense, Stroke, TextEdit, Ui, Vec2,
};

use crate::model::QuoteStatus;
use crate::theme::*;

/// Cartão branco usado como superfície das seções.
pub fn card<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add(ui)
        })
        .inner
}

/// Encurta um texto longo para caber em tabelas.
pub fn ellipsis(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        text.to_string()
    } else {
        let mut out: String = text.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

/// Cabeçalho de coluna de tabela.
pub fn th(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(10.5)
            .strong()
            .color(FAINT),
    );
}

pub fn primary_button(text: &str) -> Button<'static> {
    Button::new(RichText::new(text).color(Color32::WHITE).strong())
        .fill(PRIMARY)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(9))
        .min_size(Vec2::new(0.0, 32.0))
}

pub fn secondary_button(text: &str) -> Button<'static> {
    Button::new(RichText::new(text).color(TEXT))
        .fill(CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(9))
        .min_size(Vec2::new(0.0, 32.0))
}

pub fn small_action(ui: &mut Ui, text: &str) -> Response {
    ui.add(
        Button::new(RichText::new(text).size(12.0).color(TEXT))
            .small()
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(CornerRadius::same(7)),
    )
    .on_hover_cursor(CursorIcon::PointingHand)
}

pub fn small_danger(ui: &mut Ui, text: &str) -> Response {
    ui.add(
        Button::new(RichText::new(text).size(12.0).color(DANGER))
            .small()
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::new(1.0, Color32::from_rgb(0xF3, 0xC6, 0xC6)))
            .corner_radius(CornerRadius::same(7)),
    )
    .on_hover_cursor(CursorIcon::PointingHand)
}

pub fn small_success(ui: &mut Ui, text: &str) -> Response {
    ui.add(
        Button::new(RichText::new(text).size(12.0).color(SUCCESS).strong())
            .small()
            .fill(SUCCESS_SOFT)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(7)),
    )
    .on_hover_cursor(CursorIcon::PointingHand)
}

/// Área de busca com borda e ícone textual.
pub fn search_input(ui: &mut Ui, search: &mut String, hint: &str) -> Response {
    Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(9))
        .inner_margin(Margin::symmetric(10, 1))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
                let center = rect.center() + egui::vec2(-1.0, -1.0);
                ui.painter()
                    .circle_stroke(center, 4.2, Stroke::new(1.4, FAINT));
                ui.painter().line_segment(
                    [center + egui::vec2(3.2, 3.2), center + egui::vec2(5.8, 5.8)],
                    Stroke::new(1.4, FAINT),
                );
                ui.add(
                    TextEdit::singleline(search)
                        .hint_text(RichText::new(hint).color(FAINT))
                        .frame(Frame::NONE)
                        .margin(Margin::ZERO)
                        .desired_width(230.0),
                );
            });
        })
        .response
}

/// Permite rolagem horizontal em tabelas largas quando a janela está estreita.
pub fn table_scroll<R>(ui: &mut Ui, id: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    egui::ScrollArea::horizontal()
        .id_salt(id)
        .auto_shrink([false, true])
        .show(ui, add)
        .inner
}

/// Etiqueta colorida (status, forma de pagamento, etc.).
pub fn badge(ui: &mut Ui, text: &str, fg: Color32, bg: Color32) -> Response {
    Frame::new()
        .fill(bg)
        .corner_radius(CornerRadius::same(20))
        .inner_margin(Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.5).strong().color(fg));
        })
        .response
}

pub fn status_badge(ui: &mut Ui, status: QuoteStatus) -> Response {
    let (fg, bg) = status_colors(status);
    badge(ui, status.label(), fg, bg)
}

/// Cabeçalho grande de uma página.
pub fn page_header(ui: &mut Ui, title: &str, subtitle: &str, right_hint: &str) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(24.0).strong().color(TEXT));
            ui.add_space(2.0);
            ui.label(RichText::new(subtitle).size(12.5).color(MUTED));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(right_hint).size(12.0).color(FAINT));
        });
    });
    ui.add_space(14.0);
}

/// Cartão de indicador do painel.
pub fn stat_card(ui: &mut Ui, label: &str, value: &str, hint: &str, accent: Color32) {
    Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(15))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_min_height(84.0);
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                ui.painter().circle_filled(rect.center(), 4.0, accent);
                ui.label(RichText::new(label).size(11.5).strong().color(MUTED));
            });
            ui.add_space(6.0);
            ui.label(RichText::new(value).size(23.0).strong().color(TEXT));
            ui.add_space(2.0);
            ui.label(RichText::new(hint).size(11.0).color(FAINT));
        });
}

/// Estado vazio dentro de um cartão.
pub fn empty_state(ui: &mut Ui, title: &str, text: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(30.0);
        ui.label(RichText::new(title).size(15.0).strong().color(TEXT));
        ui.add_space(4.0);
        ui.label(RichText::new(text).size(12.5).color(MUTED));
        ui.add_space(30.0);
    });
}

/// Rótulo pequeno acima de um campo de formulário.
pub fn field_label(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).size(11.5).strong().color(MUTED));
}

/// Campo de texto de uma linha, ocupando a largura disponível.
pub fn input(ui: &mut Ui, value: &mut String, hint: &str) -> Response {
    ui.add(
        TextEdit::singleline(value)
            .hint_text(RichText::new(hint).color(FAINT))
            .margin(Margin::symmetric(10, 6))
            .desired_width(f32::INFINITY),
    )
}

/// Campo de texto multilinha.
pub fn input_multiline(ui: &mut Ui, value: &mut String, hint: &str, rows: usize) -> Response {
    ui.add(
        TextEdit::multiline(value)
            .hint_text(RichText::new(hint).color(FAINT))
            .margin(Margin::symmetric(10, 6))
            .desired_width(f32::INFINITY)
            .desired_rows(rows),
    )
}

/// Célula de grade com várias linhas.
///
/// O layout vertical normalmente quebra o texto na largura que a coluna tinha
/// no quadro anterior, impedindo a coluna de crescer. Aqui o texto usa o modo
/// `Extend`, então a largura da coluna acompanha o maior conteúdo.
pub fn grid_cell<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.vertical(|ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        add(ui)
    })
    .inner
}

/// Bloco de formulário: rótulo + campo + espaçamento.
pub fn field(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui)) {
    field_label(ui, label);
    ui.add_space(2.0);
    add(ui);
    ui.add_space(7.0);
}

/// Linha divisória sutil.
pub fn divider(ui: &mut Ui) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 1.0), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::ZERO, BORDER);
    ui.add_space(2.0);
}

/// Título de seção com ação opcional à direita.
pub fn section_title(ui: &mut Ui, title: &str) {
    ui.label(RichText::new(title).size(16.0).strong().color(TEXT));
}

/// Desenha um item do menu lateral e devolve a resposta de clique.
pub fn nav_item(ui: &mut Ui, label: &str, selected: bool) -> Response {
    let height = 40.0;
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());

    let bg = if selected {
        SIDEBAR_ACTIVE
    } else if response.hovered() {
        SIDEBAR_HOVER
    } else {
        Color32::TRANSPARENT
    };
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(9), bg);

    if selected {
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(rect.left() + 1.0, rect.center().y - 9.0),
                Vec2::new(3.0, 18.0),
            ),
            CornerRadius::same(2),
            PRIMARY,
        );
    }

    painter.circle_filled(
        egui::pos2(rect.left() + 16.0, rect.center().y),
        3.0,
        if selected { PRIMARY } else { SIDEBAR_FAINT },
    );
    painter.text(
        egui::pos2(rect.left() + 30.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(14.0),
        if selected {
            Color32::WHITE
        } else {
            SIDEBAR_TEXT
        },
    );

    response.on_hover_cursor(CursorIcon::PointingHand)
}
