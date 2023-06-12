mod bichannel;
mod standalone_app;
mod mock_api;
mod fs;

use std::{sync::Arc, borrow::Borrow};
use std::path::Path;

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

#[cfg(feature="db-mem")]
use surrealdb::engine::local::Mem as DbConnection;
#[cfg(feature="db-rocksdb")]
use surrealdb::engine::local::RocksDb as DbConnection;
use transport::ReceiveError;

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    let user_data_dir = data_local_dir().unwrap();
    let app_data_dir = user_data_dir.join("OpenXD");
    let fs = Arc::new(FileSystemStorage::new(app_data_dir.clone()));

    #[cfg(feature="db-mem")]
    let db_path = ();
    #[cfg(feature="db-rocksdb")]
    let db_path_buf = app_data_dir.join("kv.db");
    #[cfg(feature="db-rocksdb")]
    let db_path: &Path = db_path_buf.borrow();

    let db = Arc::new(Surreal::new::<DbConnection>(db_path).await.unwrap());

    db.use_ns("default").use_db("default").await.unwrap();

    let app = Arc::new(Mutex::new(App::new(db.clone())));
    let (uichannel, appchannel) = BiChannel::<Vec<u8>, Vec<u8>>::new::<Vec<u8>, Vec<u8>>();
    let fs_el = fs.clone();
    spawn(async move {
        let app = app.clone();
        let mut app_locked = app.lock().await;
        let mut session = app_locked.create_session(String::from("currentuser"), appchannel, fs_el).await.unwrap();

        loop {
            let message = session.receive_message().await;
            match message {
                Ok(message) => {
                    session.handle_message(message).await;
                }
                Err(e) => match e {
                    ReceiveError::Terminated => {
                        session.close().await;
                        break;
                    }
                    _ => {}
                },
            }
        }

    });
    let native_options = NativeOptions::default();
    run_native(
        "OpenXD",
        native_options,
        Box::new(move |cc| Box::new(StandaloneApp::new(cc, uichannel, db.clone(), fs.clone()))),
    )
    .unwrap();
}
