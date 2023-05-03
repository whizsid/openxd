use ui::ui::Ui;
use eframe::CreationContext;

use crate::{ws::{WebSocket, WebSocketError}, cache::{RemoteCache, RemoteCacheError}};

pub struct WebApp {
    ui: Ui<WebSocketError, RemoteCacheError, WebSocket, RemoteCache>
}

impl WebApp {
    pub fn new(cc: &CreationContext<'_>, ws: WebSocket) -> WebApp {
        WebApp { ui:  Ui::new(ws, RemoteCache::new()) }
    }
}

impl eframe::App for WebApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.ui.update(ctx);
    }
}
