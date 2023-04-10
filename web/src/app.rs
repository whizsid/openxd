use ui::ui::Ui;
use eframe::CreationContext;

pub struct WebApp {
    ui: Ui
}

impl WebApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        WebApp { ui:  Ui::new() }
    }
}

impl eframe::App for WebApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.ui.update(ctx);
    }
}
