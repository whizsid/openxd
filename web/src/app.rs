use ui::ui::Ui;
use eframe::CreationContext;

use crate::{ws::{WebSocket, WebSocketError}, file_uploader::{FileUploader, FileUploaderError}};

pub struct WebApp {
    ui: Ui<WebSocketError, FileUploaderError, WebSocket, FileUploader>
}

impl WebApp {
    pub fn new(cc: &CreationContext<'_>, ws: WebSocket) -> WebApp {
        WebApp { ui:  Ui::new(ws, FileUploader::new()) }
    }
}

impl eframe::App for WebApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.ui.update(ctx);
    }
}
