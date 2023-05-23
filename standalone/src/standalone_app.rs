use std::sync::Arc;

use app::external::CreateProjectUsingExistingFileError;
use eframe::{App, CreationContext};
use surrealdb::{engine::local::Db, Surreal};
use ui::ui::Ui;

use crate::{
    bichannel::{BiChannel, BiChannelError},
    fs::{FileSystemStorage, StorageError},
    mock_api::{MockApi, MockApiError},
};

pub struct StandaloneApp {
    ui: Ui<
        BiChannelError,
        MockApiError<StorageError>,
        BiChannel<Vec<u8>, Vec<u8>>,
        MockApi,
    >,
}

impl StandaloneApp {
    pub fn new(
        _cc: &CreationContext<'_>,
        internal: BiChannel<Vec<u8>, Vec<u8>>,
        db: Arc<Surreal<Db>>,
        storage: Arc<FileSystemStorage>,
    ) -> StandaloneApp {
        StandaloneApp {
            ui: Ui::new(internal, MockApi::new(db, storage)),
        }
    }
}

impl App for StandaloneApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.ui.update(ctx);
    }
}
