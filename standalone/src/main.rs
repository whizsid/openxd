mod bichannel;
mod standalone_app;
mod user_cache;
mod fs;

use std::sync::Arc;

use app::App;
use bichannel::BiChannel;
use dirs::data_local_dir;
use eframe::{run_native, NativeOptions};
use fs::FileSystemStorage;
use futures::lock::Mutex;
use simple_logger::SimpleLogger;
use standalone_app::StandaloneApp;
use surrealdb::Surreal;
use tokio::spawn;

#[cfg(feature="db-tikv")]
use surrealdb::engine::local::TiKv as DbConnection;
#[cfg(feature="db-mem")]
use surrealdb::engine::local::Mem as DbConnection;
#[cfg(feature="db-rocksdb")]
use surrealdb::engine::local::RocksDb as DbConnection;

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    let user_data_dir = data_local_dir().unwrap();
    let app_data_dir = user_data_dir.join("OpenXD");
    let fs = FileSystemStorage::new(app_data_dir.clone());

    #[cfg(feature="db-mem")]
    let db_path = ();
    #[cfg(any(feature="db-tikv", feature="db-rocksdb"))]
    let db_path = {
        let db_path_buf = app_data_dir.join("kv.db");
        let db_path_str = db_path_buf.to_str().unwrap();
        db_path_str.to_string()
    };

    let db = Surreal::new::<DbConnection>(db_path).await.unwrap();

    let app = Arc::new(Mutex::new(App::new(db, fs)));
    let (uichannel, appchannel) = BiChannel::<Vec<u8>, Vec<u8>>::new::<Vec<u8>, Vec<u8>>();
    spawn(async move {
        let app = app.clone();
        let mut app_locked = app.lock().await;
        let mut session = app_locked.init_session(String::new(), appchannel);
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
