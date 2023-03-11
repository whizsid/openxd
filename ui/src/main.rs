mod ui;
mod state;
mod app;
mod components;

#[tokio::main]
async fn main() {

    let native_options = eframe::NativeOptions::default();
    // TODO: Fetch App Name using build time config
    eframe::run_native(
        "OpenXD",
        native_options,
        Box::new(move |cc| Box::new(crate::ui::Ui::new(cc))),
    );
}
