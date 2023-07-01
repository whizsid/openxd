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
        let wgpu = cc.wgpu_render_state.as_ref().unwrap();
        WebApp {
            ui: Ui::new(&cc.egui_ctx, wgpu, client, external),
        }
    }
}

impl eframe::App for WebApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.ui.update(ctx);
    }
}
