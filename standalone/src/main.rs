mod app;

use app::StandaloneApp;
use eframe::{NativeOptions, run_native};

#[tokio::main]
async fn main() {
    let native_options = NativeOptions::default();
    run_native("OpenXD", native_options, Box::new(|cc| Box::new(StandaloneApp::new(cc)))).unwrap();
}
