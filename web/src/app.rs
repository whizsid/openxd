use ui::ui::Ui;
use eframe::CreationContext;

use crate::ws::{WebSocket, WebSocketError};

pub struct WebApp {
    ui: Ui<WebSocketError, WebSocket>
}

impl WebApp {
    pub fn new(cc: &CreationContext<'_>, ws: WebSocket) -> WebApp {
        WebApp { ui:  Ui::new(ws) }
    }
}

impl eframe::App for WebApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.ui.update(ctx);
    }
}
