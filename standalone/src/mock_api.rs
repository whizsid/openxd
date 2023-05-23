use std::{sync::Arc, fmt::Debug};

use app::external::{create_project_using_existing_file, CreateProjectUsingExistingFileError};
use async_trait::async_trait;
use surrealdb::{engine::local::Db, Surreal};
use ui::external::External;

use crate::fs::{FileSystemStorage, StorageError};

pub struct MockApi {
    db: Arc<Surreal<Db>>,
    storage: Arc<FileSystemStorage>,
}

#[derive(Debug)]
pub enum MockApiError<SE: Debug + std::error::Error + Send + Sync> {
    CreateProjectUsingExistingFile(CreateProjectUsingExistingFileError<SE>),
}

impl MockApi {
    pub fn new(db: Arc<Surreal<Db>>, storage: Arc<FileSystemStorage>) -> MockApi {
        MockApi { db, storage }
    }
}

#[async_trait]
impl External for MockApi {
    type Error = MockApiError<StorageError>;

    async fn create_project_using_existing_file(
        self: Arc<Self>,
        buf: Vec<u8>,
        file_name: String,
    ) -> Result<String, Self::Error> {
        let mut buf_reader: &[u8] = buf.as_slice();
        let userid = String::from("currentuser");
        let project = create_project_using_existing_file(
            self.db.clone(),
            self.storage.clone(),
            &mut buf_reader,
            file_name,
            userid
        )
        .await
        .map_err(MockApiError::CreateProjectUsingExistingFile)?;
        Ok(project.id.to_string())
    }
}
