use std::sync::Arc;

use eframe::CreationContext;
use futures::lock::Mutex;
use ui::{ui::Ui, client::{ClientImpl, Client}};

use crate::{
    rest_api::{RestApi, RestApiError},
    ws::{WebSocket, WebSocketError},
};

pub struct WebApp {
    ui: Ui,
}

impl WebApp {
    pub fn new(cc: &CreationContext<'_>, ws: WebSocket) -> WebApp {
        let client = Box::new(ClientImpl::new(ws));
        let external = Box::new(RestApi::new());
        let gl = cc.gl.clone().unwrap();
        WebApp {
            ui: Ui::new(&cc.egui_ctx, gl, client, external),
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
