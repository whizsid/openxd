use config::{WS_HOST, WS_PORT, WS_PATH};
use eframe::web::AppRunnerRef;
use wasm_bindgen::prelude::*;
use web_sys::{window, UrlSearchParams};
use ws::WebSocket;
use log::Level;

mod app;
mod config;
mod ws;
mod rest_api;

#[wasm_bindgen]
pub struct WebHandle {
    handle: AppRunnerRef,
}

#[wasm_bindgen]
impl WebHandle {
    #[wasm_bindgen]
    pub fn stop_web(&self) -> Result<(), wasm_bindgen::JsValue> {
        let mut app = self.handle.lock();
        app.destroy()
    }
}

#[wasm_bindgen]
pub fn init_wasm_hooks() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();
}

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[wasm_bindgen]
pub async fn start_app(canvas_id: &str) -> Result<WebHandle, wasm_bindgen::JsValue> {
    console_log::init_with_level(Level::Debug).unwrap();
    init_wasm_hooks();
    start_app_separate(canvas_id).await
}

/// Call this once from the HTML.
#[wasm_bindgen]
pub async fn start_app_separate(canvas_id: &str) -> Result<WebHandle, wasm_bindgen::JsValue> {
    let ticket = extract_ticket_id().expect("Ticket ID not provided");
    let web_options = eframe::WebOptions::default();
    let ws_url = format!("ws://{}:{}{}?ticket={}", WS_HOST, WS_PORT, WS_PATH, ticket);
    let ws_res = WebSocket::connect(&ws_url).await;
    match ws_res {
        Ok(ws) => eframe::start_web(
            canvas_id,
            web_options,
            Box::new(|cc| Box::new(crate::app::WebApp::new(cc, ws))),
        )
        .await
        .map(|handle| WebHandle { handle }),
        Err(e) => Err(JsValue::from_str(&format!("{:?}", e))),
    }
}

fn extract_ticket_id() -> Option<String> {
    let window = window().unwrap();
    let document = window.document().unwrap();
    let location = document.location().unwrap();
    let search = location.search().unwrap();
    let url_search_params = UrlSearchParams::new_with_str(&search).unwrap();
    url_search_params.get("ticket")
}
