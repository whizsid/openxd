use client::{Client, ClientTransport};
use std::{fmt::Debug, sync::Arc};
use surrealdb::{sql::Id, Connection, Surreal};

mod asset;
mod client;
pub mod external;
mod messages;
pub mod model;
pub mod oxd;
pub mod storage;

use model::Session as SessionModel;

pub struct App<D: Connection> {
    db: Arc<Surreal<D>>,
}

impl<D: Connection> App<D> {
    pub fn new(db: Arc<Surreal<D>>) -> App<D> {
        App { db }
    }

    /// Creating a user session in editor
    pub async fn create_session<E: Debug + Send, T: ClientTransport<E>>(
        &mut self,
        user_id: String,
        internal_client: T,
    ) -> Result<Session<E, T>, surrealdb::Error> {
        let session_data: SessionModel = self
            .db
            .create(SessionModel::TABLE)
            .content(SessionModel::create(Id::String(user_id)))
            .await?;
        Ok(Session::new(session_data, internal_client))
    }

    /// Accessing the internal database connection
    pub fn database(&self) -> Arc<Surreal<D>> {
        self.db.clone()
    }
}

/// A user session in a designer.
///
/// This is an intermidiate situation between ticket and a project.
/// So there can be sessions without any project. Also there can be multiple
/// tickets with the same session. Because someone can reuse the same session
/// after closing the editor without saving.
pub struct Session<TE: Debug + Send, T: ClientTransport<TE>> {
    client: Client<TE, T>,
    data: SessionModel,
}

impl<TE: Debug + Send, T: ClientTransport<TE>> Session<TE, T> {
    pub fn new(data: SessionModel, internal_client: T) -> Session<TE, T> {
        Session {
            client: Client::new(internal_client),
            data,
        }
    }

    /// Starting the session
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
