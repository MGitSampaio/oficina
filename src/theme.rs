//! Paleta de cores e estilo visual da aplicação.

use egui::style::WidgetVisuals;
use egui::{
    Color32, Context, CornerRadius, FontFamily, FontId, Shadow, Stroke, Style, TextStyle, Visuals,
};

// Fundo e superfícies
pub const BG: Color32 = Color32::from_rgb(0xF2, 0xF5, 0xF9);
pub const CARD: Color32 = Color32::WHITE;
pub const BORDER: Color32 = Color32::from_rgb(0xE2, 0xE8, 0xF0);
pub const SOFT: Color32 = Color32::from_rgb(0xF8, 0xFA, 0xFC);

// Texto — os três tons usam a mesma matiz e mantêm a proporção original de
// luminosidade, apenas escurecidos para melhorar a leitura sobre fundos claros.
pub const TEXT: Color32 = Color32::from_rgb(0x0F, 0x16, 0x23);
pub const MUTED: Color32 = Color32::from_rgb(0x4D, 0x58, 0x68);
pub const FAINT: Color32 = Color32::from_rgb(0x5F, 0x73, 0x90);

// Cores de marca / estados
pub const PRIMARY: Color32 = Color32::from_rgb(0x25, 0x63, 0xEB);
pub const PRIMARY_DARK: Color32 = Color32::from_rgb(0x1D, 0x4E, 0xD8);
pub const PRIMARY_SOFT: Color32 = Color32::from_rgb(0xDB, 0xE7, 0xFE);
pub const SUCCESS: Color32 = Color32::from_rgb(0x15, 0x80, 0x3D);
pub const SUCCESS_SOFT: Color32 = Color32::from_rgb(0xDC, 0xFC, 0xE7);
pub const WARN: Color32 = Color32::from_rgb(0xB4, 0x53, 0x09);
pub const WARN_SOFT: Color32 = Color32::from_rgb(0xFE, 0xF3, 0xC7);
pub const DANGER: Color32 = Color32::from_rgb(0xB9, 0x1C, 0x1C);
pub const DANGER_SOFT: Color32 = Color32::from_rgb(0xFE, 0xE2, 0xE2);
pub const INFO: Color32 = Color32::from_rgb(0x1D, 0x4E, 0xD8);
pub const INFO_SOFT: Color32 = Color32::from_rgb(0xDB, 0xEA, 0xFE);

// Menu lateral
pub const SIDEBAR: Color32 = Color32::from_rgb(0x12, 0x1A, 0x2B);
pub const SIDEBAR_HOVER: Color32 = Color32::from_rgb(0x1B, 0x25, 0x38);
pub const SIDEBAR_ACTIVE: Color32 = Color32::from_rgb(0x22, 0x2F, 0x47);
pub const SIDEBAR_TEXT: Color32 = Color32::from_rgb(0xAE, 0xBB, 0xD0);
pub const SIDEBAR_FAINT: Color32 = Color32::from_rgb(0x64, 0x74, 0x8E);

pub const RADIUS: u8 = 10;

pub fn apply(ctx: &Context) {
    ctx.set_theme(egui::Theme::Light);
    let mut style: Style = (*ctx.global_style()).clone();

    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(23.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(13.5, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(11.5, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(12.5, FontFamily::Monospace),
        ),
    ]
    .into();

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.menu_margin = egui::Margin::same(8);
    style.spacing.window_margin = egui::Margin::same(0);
    style.spacing.interact_size.y = 28.0;
    style.animation_time = 0.12;

    let mut visuals = Visuals::light();
    visuals.override_text_color = Some(TEXT);
    visuals.panel_fill = BG;
    visuals.window_fill = CARD;
    visuals.extreme_bg_color = Color32::WHITE;
    visuals.text_edit_bg_color = Some(Color32::WHITE);
    visuals.faint_bg_color = SOFT;
    visuals.code_bg_color = SOFT;
    visuals.window_corner_radius = CornerRadius::same(14);
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.window_shadow = Shadow {
        offset: [0, 10],
        blur: 28,
        spread: 0,
        color: Color32::from_black_alpha(28),
    };
    visuals.popup_shadow = Shadow {
        offset: [0, 6],
        blur: 20,
        spread: 0,
        color: Color32::from_black_alpha(24),
    };
    visuals.menu_corner_radius = CornerRadius::same(10);
    visuals.hyperlink_color = PRIMARY;
    visuals.selection.bg_fill = PRIMARY_SOFT;
    visuals.selection.stroke = Stroke::new(1.0, PRIMARY);

    let w = &mut visuals.widgets;
    w.noninteractive = WidgetVisuals {
        bg_fill: CARD,
        weak_bg_fill: CARD,
        bg_stroke: Stroke::new(1.0, BORDER),
        corner_radius: CornerRadius::same(RADIUS),
        fg_stroke: Stroke::new(1.0, TEXT),
        expansion: 0.0,
    };
    w.inactive = WidgetVisuals {
        bg_fill: Color32::from_rgb(0xEE, 0xF2, 0xF8),
        weak_bg_fill: CARD,
        bg_stroke: Stroke::new(1.0, BORDER),
        corner_radius: CornerRadius::same(RADIUS),
        fg_stroke: Stroke::new(1.0, TEXT),
        expansion: 0.0,
    };
    w.hovered = WidgetVisuals {
        bg_fill: PRIMARY_SOFT,
        weak_bg_fill: Color32::from_rgb(0xF3, 0xF7, 0xFF),
        bg_stroke: Stroke::new(1.0, PRIMARY),
        corner_radius: CornerRadius::same(RADIUS),
        fg_stroke: Stroke::new(1.0, PRIMARY),
        expansion: 0.0,
    };
    w.active = WidgetVisuals {
        bg_fill: PRIMARY,
        weak_bg_fill: PRIMARY_SOFT,
        bg_stroke: Stroke::new(1.0, PRIMARY_DARK),
        corner_radius: CornerRadius::same(RADIUS),
        fg_stroke: Stroke::new(1.0, Color32::WHITE),
        expansion: 0.0,
    };
    w.open = WidgetVisuals {
        bg_fill: PRIMARY_SOFT,
        weak_bg_fill: PRIMARY_SOFT,
        bg_stroke: Stroke::new(1.0, PRIMARY),
        corner_radius: CornerRadius::same(RADIUS),
        fg_stroke: Stroke::new(1.0, PRIMARY_DARK),
        expansion: 0.0,
    };

    style.visuals = visuals;
    ctx.set_global_style(style.clone());
    ctx.set_style_of(egui::Theme::Light, style);
}

/// Cores (texto, fundo) de cada situação de orçamento.
pub fn status_colors(status: crate::model::QuoteStatus) -> (Color32, Color32) {
    use crate::model::QuoteStatus::*;
    match status {
        Pendente => (WARN, WARN_SOFT),
        Aprovado => (INFO, INFO_SOFT),
        Concluido => (SUCCESS, SUCCESS_SOFT),
        Recusado => (DANGER, DANGER_SOFT),
    }
}
