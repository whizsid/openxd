use ui::ui::Ui;
use eframe::{CreationContext, App};

pub struct StandaloneApp {
    ui: Ui
}

impl StandaloneApp {
    pub fn new(_cc: &CreationContext<'_>) -> StandaloneApp {
        StandaloneApp {
            ui: Ui::new()
        }
    }
}

impl App for StandaloneApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.ui.update(ctx);
    }
}
