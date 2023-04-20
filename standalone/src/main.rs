mod bichannel;
mod standalone_app;
mod user_cache;

use std::sync::Arc;

use app::App;
use bichannel::BiChannel;
use eframe::{run_native, NativeOptions};
use futures::lock::Mutex;
use standalone_app::StandaloneApp;
use tokio::spawn;

#[tokio::main]
async fn main() {
    let app = Arc::new(Mutex::new(App::new()));
    let (uichannel, appchannel) = BiChannel::<Vec<u8>, Vec<u8>>::new::<Vec<u8>, Vec<u8>>();
    spawn(async move {
        let app = app.clone();
        let mut app_locked = app.lock().await;
        let mut session = app_locked.create_session(appchannel);
        session.start().await;
    });
    let native_options = NativeOptions::default();
    run_native(
        "OpenXD",
        native_options,
        Box::new(|cc| Box::new(StandaloneApp::new(cc, uichannel))),
    )
    .unwrap();
}
