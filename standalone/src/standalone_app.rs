use std::sync::Arc;

use eframe::{App, CreationContext};
use surrealdb::{engine::local::Db, Surreal};
use ui::{ui::Ui, client::ClientImpl};

use crate::{
    bichannel::{BiChannel, BiChannelError},
    fs::{FileSystemStorage, StorageError},
    mock_api::{MockApi, MockApiError},
};

pub struct StandaloneApp {
    ui: Ui,
}

impl StandaloneApp {
    pub fn new(
        cc: &CreationContext<'_>,
        internal: BiChannel<Vec<u8>, Vec<u8>>,
        db: Arc<Surreal<Db>>,
        storage: Arc<FileSystemStorage>,
    ) -> StandaloneApp {
        let client = ClientImpl::new(internal);
        let external = MockApi::new(db, storage);
        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap();
        StandaloneApp {
            ui: Ui::new(&cc.egui_ctx, wgpu_render_state, Box::new(client), Box::new(external)),
        }
    }
}

impl App for StandaloneApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.ui.update(ctx);
    }
}
