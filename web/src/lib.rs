use config::{WS_HOST, WS_PATH, WS_PORT};
use eframe::WebRunner;
use log::Level;
use wasm_bindgen::prelude::*;
use web_sys::{window, UrlSearchParams};
use ws::WebSocket;

mod app;
mod config;
mod rest_api;
mod ws;

/// Your handle to the web app from JavaScript.
#[derive(Clone)]
#[wasm_bindgen]
pub struct WebHandle {
    runner: WebRunner,
}

#[wasm_bindgen]
impl WebHandle {
    /// Installs a panic hook, then returns.
    #[allow(clippy::new_without_default)]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Redirect [`log`] message to `console.log` and friends:
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();

        Self {
            runner: WebRunner::new(),
        }
    }

    /// Call this once from JavaScript to start your app.
    #[wasm_bindgen]
    pub async fn start(&self, canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {

        let ticket = extract_ticket_id().expect("Ticket ID not provided");
        let web_options = eframe::WebOptions::default();
        let ws_url = format!("ws://{}:{}{}?ticket={}", WS_HOST, WS_PORT, WS_PATH, ticket);
        let ws_res = WebSocket::connect(&ws_url).await;
        match ws_res {
            Ok(ws) => self.runner.start(
                canvas_id,
                web_options,
                Box::new(|cc| Box::new(crate::app::WebApp::new(cc, ws))),
            )
            .await,
            Err(e) => Err(JsValue::from_str(&format!("{:?}", e))),
        }
    }

    // The following are optional:

    #[wasm_bindgen]
    pub fn destroy(&self) {
        self.runner.destroy();
    }

    /// The JavaScript can check whether or not your app has crashed:
    #[wasm_bindgen]
    pub fn has_panicked(&self) -> bool {
        self.runner.has_panicked()
    }

    #[wasm_bindgen]
    pub fn panic_message(&self) -> Option<String> {
        self.runner.panic_summary().map(|s| s.message())
    }

    #[wasm_bindgen]
    pub fn panic_callstack(&self) -> Option<String> {
        self.runner.panic_summary().map(|s| s.callstack())
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
pub async fn start_app(canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    console_log::init_with_level(Level::Debug).unwrap();
    init_wasm_hooks();
    let handle =  WebHandle::new();
    handle.start(canvas_id).await?;
    Ok(())
}

fn extract_ticket_id() -> Option<String> {
    let window = window().unwrap();
    let document = window.document().unwrap();
    let location = document.location().unwrap();
    let search = location.search().unwrap();
    let url_search_params = UrlSearchParams::new_with_str(&search).unwrap();
    url_search_params.get("ticket")
}
