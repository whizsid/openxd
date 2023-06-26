use eframe::CreationContext;
use ui::ui::Ui;

use crate::{
    rest_api::{RestApi, RestApiError},
    ws::{WebSocket, WebSocketError},
};

pub struct WebApp {
    ui: Ui<WebSocketError, RestApiError, WebSocket, RestApi>,
}

impl WebApp {
    pub fn new(cc: &CreationContext<'_>, ws: WebSocket) -> WebApp {
        let gl = cc.gl.clone().unwrap();
        WebApp {
            ui: Ui::new(&cc.egui_ctx, gl, ws, RestApi::new()),
        }
    }
}

impl eframe::App for WebApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.ui.update(ctx);
    }

    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        self.ui.exit(gl);
    }
}
