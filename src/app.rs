//! Estado da aplicação e janela principal.

use std::time::{Duration, Instant};

use eframe::CreationContext;
use egui::{
    Align, Align2, Color32, Context, CornerRadius, Frame, Id, Layout, Margin, Order, RichText,
    Stroke, Ui,
};

use crate::db::Store;
use crate::model::{Client, Equipment, Payment, Quote, QuoteStatus, Stats};
use crate::theme::*;
use crate::ui::forms::{ClientForm, DialogOutcome, EquipmentForm, PaymentForm, QuoteForm};
use crate::ui::widgets as w;
use crate::util::{current_month_prefix, today_long_br};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Clients,
    Equipment,
    Quotes,
    Payments,
}

impl Page {
    fn hero(self) -> (&'static str, &'static str) {
        match self {
            Page::Dashboard => (
                "Painel",
                "Visão geral da oficina, recebimentos e serviços em andamento",
            ),
            Page::Clients => (
                "Clientes",
                "Cadastro de clientes e histórico de atendimentos",
            ),
            Page::Equipment => (
                "Equipamentos",
                "Aparelhos recebidos para conserto e seus responsáveis",
            ),
            Page::Quotes => (
                "Orçamentos",
                "Serviços, peças, valores e situação de cada conserto",
            ),
            Page::Payments => (
                "Pagamentos",
                "Entradas registradas e saldo em aberto dos orçamentos",
            ),
        }
    }
}

pub enum Dialog {
    None,
    Client(ClientForm),
    Equipment(EquipmentForm),
    Quote(QuoteForm),
    Payment(PaymentForm),
    Confirm(ConfirmState),
}

pub struct ConfirmState {
    pub title: String,
    pub message: String,
    pub target: DeleteTarget,
}

#[derive(Clone, Copy)]
pub enum DeleteTarget {
    Client(i64),
    Equipment(i64),
    Quote(i64),
    Payment(i64),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Error,
}

pub struct Toast {
    pub text: String,
    pub kind: ToastKind,
    pub born: Instant,
}

pub struct App {
    pub(crate) store: Store,
    pub(crate) page: Page,
    pub(crate) search: String,

    pub(crate) clients: Vec<Client>,
    pub(crate) equipments: Vec<Equipment>,
    pub(crate) quotes: Vec<Quote>,
    pub(crate) payments: Vec<Payment>,
    pub(crate) stats: Stats,

    pub(crate) status_filter: Option<QuoteStatus>,
    pub(crate) equipment_client_filter: Option<i64>,

    pub(crate) dialog: Dialog,
    pub(crate) toast: Option<Toast>,
}

impl App {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        let store = Store::open_default().unwrap_or_else(|err| {
            panic!(
                "Não foi possível abrir o banco de dados da oficina.\nDetalhe: {err}\n\
                 Verifique as permissões da pasta de dados do usuário."
            )
        });
        Self::with_store(store)
    }

    /// Cria a aplicação a partir de um banco já aberto (usado também nos testes).
    pub fn with_store(store: Store) -> Self {
        let mut app = Self {
            store,
            page: Page::Dashboard,
            search: String::new(),
            clients: Vec::new(),
            equipments: Vec::new(),
            quotes: Vec::new(),
            payments: Vec::new(),
            stats: Stats::default(),
            status_filter: None,
            equipment_client_filter: None,
            dialog: Dialog::None,
            toast: None,
        };
        app.reload();
        app
    }

    /// Recarrega todos os dados do banco.
    pub fn reload(&mut self) {
        let mut error: Option<String> = None;

        match self.store.list_clients() {
            Ok(rows) => self.clients = rows,
            Err(err) => error = Some(err),
        }
        match self.store.list_equipment() {
            Ok(rows) => self.equipments = rows,
            Err(err) => error = Some(err),
        }
        match self.store.list_quotes() {
            Ok(rows) => self.quotes = rows,
            Err(err) => error = Some(err),
        }
        match self.store.list_payments() {
            Ok(rows) => self.payments = rows,
            Err(err) => error = Some(err),
        }

        self.recalc_stats();

        if let Some(err) = error {
            self.notify_error(format!("Erro ao carregar dados: {err}"));
        }
    }

    fn recalc_stats(&mut self) {
        let mut stats = Stats {
            clients: self.clients.len(),
            equipment: self.equipments.len(),
            ..Stats::default()
        };

        for quote in &self.quotes {
            if quote.status.is_open() {
                stats.open_quotes += 1;
            }
            if quote.status != QuoteStatus::Recusado && quote.balance_cents() > 0 {
                stats.payable_quotes += 1;
                stats.receivable_cents += quote.balance_cents();
            }
        }

        let month = current_month_prefix();
        for payment in &self.payments {
            stats.total_revenue_cents += payment.amount_cents;
            if payment.paid_on.starts_with(&month) {
                stats.month_revenue_cents += payment.amount_cents;
            }
        }

        self.stats = stats;
    }

    pub fn notify_success(&mut self, text: impl Into<String>) {
        self.toast = Some(Toast {
            text: text.into(),
            kind: ToastKind::Success,
            born: Instant::now(),
        });
    }

    pub fn notify_error(&mut self, text: impl Into<String>) {
        self.toast = Some(Toast {
            text: text.into(),
            kind: ToastKind::Error,
            born: Instant::now(),
        });
    }

    // ------------------------------------------------------------------ Formulários

    pub fn open_client_form(&mut self, id: Option<i64>) {
        let form = match id.and_then(|id| self.clients.iter().find(|c| c.id == id)) {
            Some(client) => ClientForm::edit(client),
            None => ClientForm::new(),
        };
        self.dialog = Dialog::Client(form);
    }

    pub fn open_equipment_form(&mut self, id: Option<i64>) {
        let form = match id.and_then(|id| self.equipments.iter().find(|e| e.id == id)) {
            Some(equipment) => EquipmentForm::edit(equipment),
            None => EquipmentForm::new(self.equipment_client_filter),
        };
        self.dialog = Dialog::Equipment(form);
    }

    pub fn open_quote_form(&mut self, id: Option<i64>) {
        let form = match id.and_then(|id| self.quotes.iter().find(|q| q.id == id)) {
            Some(quote) => QuoteForm::edit(quote),
            None => QuoteForm::new(),
        };
        self.dialog = Dialog::Quote(form);
    }

    pub fn open_payment_form(&mut self, id: Option<i64>) {
        let form = match id.and_then(|id| self.payments.iter().find(|p| p.id == id)) {
            Some(payment) => PaymentForm::edit(payment),
            None => PaymentForm::new(),
        };
        self.dialog = Dialog::Payment(form);
    }

    /// Abre o formulário de pagamento já apontando para um orçamento.
    pub fn open_payment_for_quote(&mut self, quote_id: i64) {
        let mut form = PaymentForm::new();
        form.quote_id = Some(quote_id);
        if let Some(quote) = self.quotes.iter().find(|q| q.id == quote_id) {
            let balance = quote.balance_cents();
            if balance > 0 {
                form.amount = crate::util::money_plain(balance);
            }
        }
        self.dialog = Dialog::Payment(form);
    }

    /// Gera a versão para impressão do orçamento e abre no navegador padrão.
    pub fn print_quote(&mut self, quote_id: i64) {
        let Some(quote) = self.quotes.iter().find(|q| q.id == quote_id) else {
            self.notify_error("Orçamento não encontrado.");
            return;
        };
        let client = self.clients.iter().find(|c| c.id == quote.client_id);
        let equipment = quote
            .equipment_id
            .and_then(|id| self.equipments.iter().find(|e| e.id == id));
        let payments: Vec<_> = self
            .payments
            .iter()
            .filter(|payment| payment.quote_id == quote_id)
            .cloned()
            .collect();

        match crate::print::quote(quote, client, equipment, &payments) {
            Ok(_) => self.notify_success("Orçamento aberto no navegador para impressão."),
            Err(err) => self.notify_error(err),
        }
    }

    pub fn ask_delete(&mut self, target: DeleteTarget) {
        let (title, message) = match target {
            DeleteTarget::Client(id) => {
                let name = self
                    .clients
                    .iter()
                    .find(|c| c.id == id)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "este cliente".to_string());
                (
                    "Excluir cliente?".to_string(),
                    format!(
                        "“{name}” será removido junto com seus equipamentos, orçamentos e pagamentos. \
                         Esta ação não pode ser desfeita."
                    ),
                )
            }
            DeleteTarget::Equipment(id) => {
                let name = self
                    .equipments
                    .iter()
                    .find(|e| e.id == id)
                    .map(|e| e.title())
                    .unwrap_or_else(|| "este equipamento".to_string());
                (
                    "Excluir equipamento?".to_string(),
                    format!(
                        "“{name}” será removido. Os orçamentos existentes continuarão salvos, \
                         mas ficarão sem equipamento associado."
                    ),
                )
            }
            DeleteTarget::Quote(id) => {
                let summary = self
                    .quotes
                    .iter()
                    .find(|q| q.id == id)
                    .map(|q| q.summary())
                    .unwrap_or_else(|| "este orçamento".to_string());
                (
                    "Excluir orçamento?".to_string(),
                    format!(
                        "O orçamento “{summary}” e todos os pagamentos registrados nele serão \
                         removidos definitivamente."
                    ),
                )
            }
            DeleteTarget::Payment(id) => {
                let amount = self
                    .payments
                    .iter()
                    .find(|p| p.id == id)
                    .map(|p| crate::util::money(p.amount_cents))
                    .unwrap_or_else(|| "este pagamento".to_string());
                (
                    "Excluir pagamento?".to_string(),
                    format!(
                        "O pagamento de {amount} será removido e deixará de contar no saldo do \
                         orçamento."
                    ),
                )
            }
        };

        self.dialog = Dialog::Confirm(ConfirmState {
            title,
            message,
            target,
        });
    }

    fn save_dialog(&mut self, dialog: Dialog) {
        match dialog {
            Dialog::Client(mut form) => match form.save(&self.store) {
                Ok(message) => {
                    self.dialog = Dialog::None;
                    self.reload();
                    self.notify_success(message);
                }
                Err(err) => {
                    form.error = Some(err);
                    self.dialog = Dialog::Client(form);
                }
            },
            Dialog::Equipment(mut form) => match form.save(&self.store) {
                Ok(message) => {
                    self.dialog = Dialog::None;
                    self.reload();
                    self.notify_success(message);
                }
                Err(err) => {
                    form.error = Some(err);
                    self.dialog = Dialog::Equipment(form);
                }
            },
            Dialog::Quote(mut form) => match form.save(&self.store) {
                Ok(message) => {
                    self.dialog = Dialog::None;
                    self.reload();
                    self.notify_success(message);
                }
                Err(err) => {
                    form.error = Some(err);
                    self.dialog = Dialog::Quote(form);
                }
            },
            Dialog::Payment(mut form) => match form.save(&self.store) {
                Ok(message) => {
                    self.dialog = Dialog::None;
                    self.reload();
                    self.notify_success(message);
                }
                Err(err) => {
                    form.error = Some(err);
                    self.dialog = Dialog::Payment(form);
                }
            },
            Dialog::Confirm(state) => {
                self.perform_delete(state.target);
            }
            Dialog::None => {}
        }
    }

    fn perform_delete(&mut self, target: DeleteTarget) {
        let result = match target {
            DeleteTarget::Client(id) => self.store.delete_client(id),
            DeleteTarget::Equipment(id) => self.store.delete_equipment(id),
            DeleteTarget::Quote(id) => self.store.delete_quote(id),
            DeleteTarget::Payment(id) => self.store.delete_payment(id),
        };

        match result {
            Ok(()) => {
                self.reload();
                self.notify_success("Registro excluído.");
            }
            Err(err) => self.notify_error(format!("Não foi possível excluir: {err}")),
        }
    }

    // ------------------------------------------------------------------ Layout

    fn sidebar(&mut self, ui: &mut Ui) {
        egui::Panel::left("sidebar")
            .resizable(false)
            .exact_size(226.0)
            .frame(Frame::new().fill(SIDEBAR).inner_margin(Margin {
                left: 14,
                right: 14,
                top: 20,
                bottom: 16,
            }))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(34.0, 34.0), egui::Sense::hover());
                    ui.painter()
                        .rect_filled(rect, CornerRadius::same(10), PRIMARY);
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        "O",
                        egui::FontId::proportional(19.0),
                        Color32::WHITE,
                    );
                    ui.add_space(2.0);
                    ui.vertical(|ui| {
                        ui.add_space(1.0);
                        ui.label(
                            RichText::new("Oficina")
                                .size(17.0)
                                .strong()
                                .color(Color32::WHITE),
                        );
                        ui.label(
                            RichText::new("Assistência técnica")
                                .size(10.5)
                                .color(SIDEBAR_TEXT),
                        );
                    });
                });

                ui.add_space(26.0);
                ui.label(
                    RichText::new("MENU")
                        .size(9.5)
                        .strong()
                        .color(SIDEBAR_FAINT),
                );
                ui.add_space(8.0);

                let items = [
                    (Page::Dashboard, "Painel"),
                    (Page::Clients, "Clientes"),
                    (Page::Equipment, "Equipamentos"),
                    (Page::Quotes, "Orçamentos"),
                    (Page::Payments, "Pagamentos"),
                ];
                for (page, label) in items {
                    if w::nav_item(ui, label, self.page == page).clicked() {
                        self.page = page;
                        self.search.clear();
                    }
                    ui.add_space(2.0);
                }

                let remaining = ui.available_height();
                ui.add_space((remaining - 84.0).max(0.0));

                let (line, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 1.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(
                    line,
                    CornerRadius::ZERO,
                    Color32::from_rgb(0x26, 0x32, 0x48),
                );
                ui.add_space(10.0);

                ui.label(
                    RichText::new("BANCO DE DADOS")
                        .size(9.0)
                        .strong()
                        .color(SIDEBAR_FAINT),
                );
                ui.add_space(3.0);
                let file_name = self
                    .store
                    .path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| "oficina.db".to_string());
                ui.label(RichText::new(file_name).size(10.0).color(SIDEBAR_TEXT))
                    .on_hover_text(self.store.path.display().to_string());
                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!("Oficina v{}", env!("CARGO_PKG_VERSION")))
                        .size(9.5)
                        .color(SIDEBAR_FAINT),
                );
            });
    }

    fn show_dialogs(&mut self, ctx: &Context) {
        if matches!(self.dialog, Dialog::None) {
            return;
        }

        let mut dialog = std::mem::replace(&mut self.dialog, Dialog::None);
        let mut outcome = DialogOutcome::Keep;

        {
            let clients = &self.clients;
            let equipments = &self.equipments;
            let quotes = &self.quotes;

            match &mut dialog {
                Dialog::Client(form) => {
                    let title = form.title();
                    let response = modal_shell(ctx, "dialog_client", 540.0, |ui| {
                        ui.label(RichText::new(title).size(19.0).strong());
                        ui.add_space(10.0);
                        form.show(ui)
                    });
                    match response {
                        Some(action) => outcome = action,
                        None => outcome = DialogOutcome::Cancel,
                    }
                }
                Dialog::Equipment(form) => {
                    let title = form.title();
                    let response = modal_shell(ctx, "dialog_equipment", 600.0, |ui| {
                        ui.label(RichText::new(title).size(19.0).strong());
                        ui.add_space(10.0);
                        form.show(ui, clients)
                    });
                    match response {
                        Some(action) => outcome = action,
                        None => outcome = DialogOutcome::Cancel,
                    }
                }
                Dialog::Quote(form) => {
                    let title = form.title();
                    let response = modal_shell(ctx, "dialog_quote", 660.0, |ui| {
                        ui.label(RichText::new(title).size(19.0).strong());
                        ui.add_space(10.0);
                        form.show(ui, clients, equipments)
                    });
                    match response {
                        Some(action) => outcome = action,
                        None => outcome = DialogOutcome::Cancel,
                    }
                }
                Dialog::Payment(form) => {
                    let title = form.title();
                    let response = modal_shell(ctx, "dialog_payment", 600.0, |ui| {
                        ui.label(RichText::new(title).size(19.0).strong());
                        ui.add_space(10.0);
                        form.show(ui, quotes)
                    });
                    match response {
                        Some(action) => outcome = action,
                        None => outcome = DialogOutcome::Cancel,
                    }
                }
                Dialog::Confirm(state) => {
                    let mut action = DialogOutcome::Keep;
                    let modal = egui::Modal::new(Id::unique("dialog_confirm"))
                        .frame(dialog_frame())
                        .show(ctx, |ui| {
                            ui.set_width(410.0);
                            ui.label(RichText::new(&state.title).size(19.0).strong());
                            ui.add_space(10.0);
                            ui.label(RichText::new(&state.message).size(13.0).color(MUTED));
                            ui.add_space(18.0);
                            ui.horizontal(|ui| {
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let danger = egui::Button::new(
                                        RichText::new("Excluir").color(Color32::WHITE).strong(),
                                    )
                                    .fill(DANGER)
                                    .stroke(Stroke::NONE)
                                    .corner_radius(CornerRadius::same(9))
                                    .min_size(egui::vec2(96.0, 32.0));
                                    if ui.add(danger).clicked() {
                                        action = DialogOutcome::Save;
                                    }
                                    if ui.add(w::secondary_button("Cancelar")).clicked() {
                                        action = DialogOutcome::Cancel;
                                    }
                                });
                            });
                        });
                    if modal.should_close() {
                        action = DialogOutcome::Cancel;
                    }
                    outcome = action;
                }
                Dialog::None => {}
            }
        }

        match outcome {
            DialogOutcome::Keep => self.dialog = dialog,
            DialogOutcome::Save => self.save_dialog(dialog),
            DialogOutcome::Cancel => self.dialog = Dialog::None,
        }
    }

    fn show_toast(&mut self, ctx: &Context) {
        let Some(toast) = &self.toast else {
            return;
        };
        let elapsed = toast.born.elapsed();
        if elapsed > Duration::from_secs(5) {
            self.toast = None;
            return;
        }

        let fade = if elapsed > Duration::from_secs_f32(4.2) {
            1.0 - ((elapsed.as_secs_f32() - 4.2) / 0.8).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let (bg, dot) = match toast.kind {
            ToastKind::Success => (
                Color32::from_rgb(0x0C, 0x3B, 0x2C),
                Color32::from_rgb(0x4A, 0xDE, 0x80),
            ),
            ToastKind::Error => (
                Color32::from_rgb(0x6B, 0x1B, 0x1B),
                Color32::from_rgb(0xFC, 0xA5, 0xA5),
            ),
        };

        egui::Area::new(Id::unique("toast"))
            .anchor(Align2::RIGHT_BOTTOM, egui::vec2(-24.0, -24.0))
            .order(Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                Frame::new()
                    .fill(bg.gamma_multiply(fade))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::symmetric(16, 11))
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 6],
                        blur: 18,
                        spread: 0,
                        color: Color32::from_black_alpha((70.0 * fade) as u8),
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (rect, _) =
                                ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                            ui.painter().circle_filled(rect.center(), 4.0, dot);
                            ui.label(RichText::new(&toast.text).size(12.5).color(Color32::WHITE));
                        });
                    });
            });

        ctx.request_repaint_after(Duration::from_millis(100));
    }

    /// Desenha a aplicação inteira dentro de um `Ui` raiz.
    pub fn show(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        self.sidebar(ui);

        egui::CentralPanel::default()
            .frame(Frame::new().fill(BG).inner_margin(Margin {
                left: 24,
                right: 24,
                top: 16,
                bottom: 16,
            }))
            .show(ui, |ui| {
                let page = self.page;
                let (title, subtitle) = page.hero();
                let today = today_long_br();
                w::page_header(ui, title, subtitle, &today);

                match page {
                    Page::Dashboard => self.page_dashboard(ui),
                    Page::Clients => self.page_clients(ui),
                    Page::Equipment => self.page_equipment(ui),
                    Page::Quotes => self.page_quotes(ui),
                    Page::Payments => self.page_payments(ui),
                }
            });

        self.show_dialogs(&ctx);
        self.show_toast(&ctx);
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

/// Janela modal padrão com cartão arredondado. Devolve `None` se fechada.
fn modal_shell<T>(
    ctx: &Context,
    id: &str,
    width: f32,
    add: impl FnOnce(&mut Ui) -> T,
) -> Option<T> {
    let area_id = Id::unique(id);
    // Moldura (22 px de cada lado) + um respiro curto até a borda da janela.
    let max_height = (ctx.content_rect().height() - 68.0).max(280.0);
    // O tamanho inicial do `Area` precisa já comportar o conteúdo; caso contrário
    // o `ScrollArea` limita a altura na primeira medição e a área nunca cresce,
    // deixando a barra de rolagem visível mesmo quando o formulário caberia.
    let initial_size = egui::vec2(width + 44.0, max_height + 44.0);
    let modal = egui::Modal::new(area_id)
        .area(egui::Modal::default_area(area_id).default_size(initial_size))
        .frame(dialog_frame())
        .show(ctx, |ui| {
            ui.set_width(width);
            // Só rola quando a janela for realmente baixa demais.
            egui::ScrollArea::vertical()
                .max_height(max_height)
                .auto_shrink([false, true])
                .show(ui, add)
                .inner
        });
    if modal.should_close() {
        None
    } else {
        Some(modal.inner)
    }
}

fn dialog_frame() -> Frame {
    Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(Margin::same(22))
        .shadow(egui::epaint::Shadow {
            offset: [0, 14],
            blur: 40,
            spread: 0,
            color: Color32::from_black_alpha(48),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Executa quadros do egui sem janela, exercitando o layout de verdade.
    fn render(ctx: &Context, app: &mut App, size: egui::Vec2) {
        for _ in 0..2 {
            let mut input = egui::RawInput::default();
            input.screen_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size));
            let mut output = ctx.run_ui(input, |ui| {
                app.show(ui);
            });
            // Sem backend gráfico, as texturas geradas não têm para onde ir.
            output.textures_delta.clear();
        }
    }

    #[test]
    fn all_pages_and_dialogs_render() {
        let store = crate::db::Store::in_memory().expect("banco em memória");
        store.seed_demo().expect("dados de exemplo");

        let ctx = Context::default();
        crate::theme::apply(&ctx);
        let mut app = App::with_store(store);

        let pages = [
            Page::Dashboard,
            Page::Clients,
            Page::Equipment,
            Page::Quotes,
            Page::Payments,
        ];
        for page in pages {
            app.page = page;
            render(&ctx, &mut app, egui::vec2(1220.0, 790.0));
            render(&ctx, &mut app, egui::vec2(1020.0, 640.0));
        }

        app.open_client_form(None);
        render(&ctx, &mut app, egui::vec2(1020.0, 640.0));
        app.dialog = Dialog::None;

        app.open_equipment_form(None);
        render(&ctx, &mut app, egui::vec2(1020.0, 640.0));
        app.dialog = Dialog::None;

        app.open_quote_form(None);
        render(&ctx, &mut app, egui::vec2(1020.0, 640.0));
        app.dialog = Dialog::None;

        app.open_payment_form(None);
        render(&ctx, &mut app, egui::vec2(1020.0, 640.0));
        app.dialog = Dialog::None;

        app.ask_delete(DeleteTarget::Client(app.clients[0].id));
        render(&ctx, &mut app, egui::vec2(1020.0, 640.0));
    }
}
