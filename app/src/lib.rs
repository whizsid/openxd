use cache::{Cache, CacheFileError};
use std::{fmt::Debug, sync::Arc};
use storage::{Storage, StorageId};
use surrealdb::{Connection, Surreal};
use tokio::io::AsyncBufRead;
use uuid::Uuid;
use client::{Client, ClientTransport};

mod asset;
pub mod cache;
mod client;
mod messages;
pub mod oxd;
pub mod storage;

pub struct App<SI: StorageId, SE: Debug, D: Connection, S: Storage<SE, SI>> {
    cache: Cache<SI, SE, D, S>,
    db: Arc<Surreal<D>>
}

impl<SI: StorageId, SE: Debug, D: Connection, S: Storage<SE, SI>> App<SI, SE, D, S> {
    pub fn new(db: Surreal<D>, storage: S) -> App<SI, SE, D, S> {
        let rc_db = Arc::new(db);
        App {
            db: rc_db.clone(),
            cache: Cache::new(rc_db, storage),
        }
    }

    pub async fn create_session_with_file<R: AsyncBufRead + Send + Sync + Unpin> (&self, content: R) -> Result<Uuid, CacheFileError<SE>> {
        let session_id = Uuid::new_v4();
        self.cache.cache_file(session_id, content).await?;
        Ok(session_id)
    }

    pub fn init_session<
        E: Debug + Send,
        T: ClientTransport<E>
    >(
        &mut self,
        internal_client: T,
    ) -> Session<E, T> {
        Session::new(internal_client)
    }

    /// Accessing the internal database connection
    pub fn database(&self) -> Arc<Surreal<D>> {
        self.db.clone()
    }
}

pub struct Session<TE: Debug + Send, T: ClientTransport<TE>> {
    client: Client<TE, T>,
    id: Option<Uuid>,
}

impl<TE: Debug + Send, T: ClientTransport<TE>> Session<TE, T> {
    pub fn new(internal_client: T) -> Session<TE, T> {
        Session {
            client: Client::new(internal_client),
            id: None
        }
    }

    pub async fn start(&mut self) {
        let start_message_res = self.client.wait_till_init().await;
        match start_message_res {
            Ok(start_message) => {
                self.client.file_opened().await.unwrap();
            }
            Err(start_err) => {
                self.client.error(start_err).await.unwrap();
            }
        }
    }
}
