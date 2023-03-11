#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};
#[cfg(target_arch = "wasm32")]
use eframe::web::AppRunnerRef;

mod app;
mod state;
mod ui;
mod components;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WebHandle {
    handle: AppRunnerRef,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WebHandle {
    #[wasm_bindgen]
    pub fn stop_web(&self) -> Result<(), wasm_bindgen::JsValue> {
        let mut app = self.handle.lock();
        app.destroy()
    }
}

#[cfg(target_arch = "wasm32")]
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
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start_app(canvas_id: &str) -> Result<WebHandle, wasm_bindgen::JsValue> {
    init_wasm_hooks();
    start_app_separate(canvas_id).await
}

/// Call this once from the HTML.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start_app_separate(
    canvas_id: &str,
) -> Result<WebHandle, wasm_bindgen::JsValue> {
    let web_options = eframe::WebOptions::default();
    eframe::start_web(
        canvas_id,
        web_options,
        Box::new(|cc| Box::new(crate::ui::Ui::new(cc))),
    )
    .await
    .map(|handle| WebHandle { handle })
}
