use ui::ui::Ui;
use eframe::{CreationContext, App};

use crate::{bichannel::{BiChannelError, BiChannel}, user_cache::{UserCacheError, UserCache}};

pub struct StandaloneApp {
    ui: Ui<BiChannelError, UserCacheError, BiChannel<Vec<u8>, Vec<u8>>, UserCache>
}

impl StandaloneApp {
    pub fn new(_cc: &CreationContext<'_>, internal: BiChannel<Vec<u8>, Vec<u8>>) -> StandaloneApp {
        StandaloneApp {
            ui: Ui::new(internal, UserCache::new())
        }
    }
}

impl App for StandaloneApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.ui.update(ctx);
    }
}
