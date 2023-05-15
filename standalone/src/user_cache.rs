use std::sync::Arc;

use app::external::{create_project_using_existing_file, CreateProjectUsingExistingFileError};
use async_trait::async_trait;
use surrealdb::{engine::local::Db, Surreal};
use ui::cache::Cache as UICache;

use crate::fs::{FileSystemStorage, StorageError};

pub struct UserCache {
    db: Arc<Surreal<Db>>,
    storage: Arc<FileSystemStorage>,
}

impl UserCache {
    pub fn new(db: Arc<Surreal<Db>>, storage: Arc<FileSystemStorage>) -> UserCache {
        UserCache { db, storage }
    }
}

#[async_trait]
impl UICache for UserCache {
    type Error = CreateProjectUsingExistingFileError<StorageError>;

    async fn cache_file(self: Arc<Self>, buf: Vec<u8>) -> Result<String, Self::Error> {
        let mut buf_reader: &[u8] = buf.as_slice();
        let project = create_project_using_existing_file(self.db.clone(), self.storage.clone(), &mut buf_reader).await?;
        Ok(project.id.to_string())
    }
}
