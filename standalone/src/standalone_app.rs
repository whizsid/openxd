use ui::ui::Ui;
use eframe::{CreationContext, App};

use crate::bichannel::{BiChannelError, BiChannel};

pub struct StandaloneApp {
    ui: Ui<BiChannelError, BiChannel<Vec<u8>, Vec<u8>>>
}

impl StandaloneApp {
    pub fn new(_cc: &CreationContext<'_>, internal: BiChannel<Vec<u8>, Vec<u8>>) -> StandaloneApp {
        StandaloneApp {
            ui: Ui::new(internal)
        }
    }
}

impl App for StandaloneApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.ui.update(ctx);
    }
}
