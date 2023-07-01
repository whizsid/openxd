mod bichannel;
mod fs;
mod mock_api;
mod standalone_app;

use std::path::Path;
use std::{borrow::Borrow, sync::Arc};

use app::model::User;
use app::App;
use bichannel::BiChannel;
use dirs::data_local_dir;
use eframe::{run_native, NativeOptions};
use fs::FileSystemStorage;
use futures::lock::Mutex;
use log::{debug, info};
use standalone_app::StandaloneApp;
use surrealdb::Surreal;
use tokio::spawn;

#[cfg(feature = "db-mem")]
use surrealdb::engine::local::Mem as DbConnection;
#[cfg(feature = "db-rocksdb")]
use surrealdb::engine::local::RocksDb as DbConnection;
use transport::ReceiveError;

pub static USER_ID: &str = "currentuser";

#[tokio::main]
async fn main() {
    simple_logger::init_with_env().unwrap();

    let user_data_dir = data_local_dir().unwrap();
    let app_data_dir = user_data_dir.join("OpenXD");
    let fs = Arc::new(FileSystemStorage::new(app_data_dir.clone()));

    #[cfg(feature = "db-mem")]
    let db_path = ();
    #[cfg(feature = "db-rocksdb")]
    let db_path_buf = app_data_dir.join("kv.db");
    #[cfg(feature = "db-rocksdb")]
    let db_path: &Path = db_path_buf.borrow();

    let db = Arc::new(Surreal::new::<DbConnection>(db_path).await.unwrap());

    db.use_ns("default").use_db("default").await.unwrap();

    let mut exist_user: Option<User> = db.select((User::TABLE, USER_ID)).await.unwrap();
    if exist_user.is_none() {
        exist_user = Some(
            db.create((User::TABLE, USER_ID))
                .content(User::new(String::from("Root")))
                .await
                .unwrap(),
        );
    }

    let app = Arc::new(Mutex::new(App::new(db.clone())));
    let (uichannel, appchannel) = BiChannel::<Vec<u8>, Vec<u8>>::new::<Vec<u8>, Vec<u8>>();
    let fs_el = fs.clone();
    spawn(async move {
        let app = app.clone();
        let mut app_locked = app.lock().await;
        let mut session = app_locked
            .create_session(String::from(USER_ID), appchannel, fs_el)
            .await
            .unwrap();

        loop {
            let message = session.receive_message().await;
            debug!("{:?}", &message);
            match message {
                Ok(message) => {
                    session.handle_message(message).await;
                }
                Err(e) => match e {
                    ReceiveError::Terminated => {
                        session.close().await;
                        info!("Connection Terminated!");
                        break;
                    }
                    _ => {}
                },
            }
        }
    });
    let options = NativeOptions {
        multisampling: 4,
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    run_native(
        "OpenXD",
        options,
        Box::new(move |cc| Box::new(StandaloneApp::new(cc, uichannel, db.clone(), fs.clone()))),
    )
    .unwrap();
}
