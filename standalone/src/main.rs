mod standalone_app;
mod bichannel;

use app::Session;
use standalone_app::StandaloneApp;
use bichannel::BiChannel;
use eframe::{NativeOptions, run_native};
use tokio::spawn;

#[tokio::main]
async fn main() {
    let (uichannel, appchannel) = BiChannel::<Vec<u8>, Vec<u8>>::new::<Vec<u8>, Vec<u8>>();
    spawn(async move {
        let mut session = Session::new(appchannel);
        session.start().await;
    });
    let native_options = NativeOptions::default();
    run_native("OpenXD", native_options, Box::new(|cc| Box::new(StandaloneApp::new(cc, uichannel)))).unwrap();
}
