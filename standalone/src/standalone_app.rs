use std::sync::Arc;

use eframe::{App, CreationContext};
use surrealdb::{engine::local::Db, Surreal};
use ui::ui::Ui;

use crate::{
    bichannel::{BiChannel, BiChannelError},
    fs::{FileSystemStorage, StorageError},
    mock_api::{MockApi, MockApiError},
};

pub struct StandaloneApp {
    ui: Ui<BiChannelError, MockApiError<StorageError>, BiChannel<Vec<u8>, Vec<u8>>, MockApi>,
}

impl StandaloneApp {
    pub fn new(
        cc: &CreationContext<'_>,
        internal: BiChannel<Vec<u8>, Vec<u8>>,
        db: Arc<Surreal<Db>>,
        storage: Arc<FileSystemStorage>,
    ) -> StandaloneApp {
        let gl = cc.gl.clone().unwrap();
        StandaloneApp {
            ui: Ui::new(&cc.egui_ctx, gl, internal, MockApi::new(db, storage)),
        }
    }
}

impl App for StandaloneApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.ui.update(ctx);
    }

    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        self.ui.exit(gl);
    }
}
