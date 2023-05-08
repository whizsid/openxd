use std::path::PathBuf;

use app::storage::Storage;
use async_trait::async_trait;
use tokio::{
    fs::{remove_file, File},
    io::{copy, AsyncRead},
};
use uuid::Uuid;

use crate::config::STORAGE_FS_ROOT;

pub struct FileSystemStorage;

#[derive(Debug)]
pub enum StorageError {
    Io(std::io::Error),
}

#[async_trait]
impl Storage<StorageError, PathBuf> for FileSystemStorage {
    type Read = File;
    /// Saving a file to storage
    ///
    /// Namespace is used to identify the type of the file
    async fn put<'a, I: AsyncRead + Unpin + Send>(
        &self,
        reader: &'a mut I,
        namespace: String,
        ext: String,
    ) -> Result<PathBuf, StorageError> {
        let file_name = Uuid::new_v4();
        let path = PathBuf::new()
            .join(STORAGE_FS_ROOT)
            .join(namespace)
            .join(format!("{}.{}", file_name, ext));
        let mut file = File::open(path.clone())
            .await
            .map_err(|e| StorageError::Io(e))?;
        copy(reader, &mut file)
            .await
            .map_err(|e| StorageError::Io(e))?;
        Ok(path.to_path_buf())
    }

    /// Retrieving the saved file from the storage
    async fn get(&self, key: PathBuf) -> Result<Self::Read, StorageError> {
        let file = File::open(key).await.map_err(|e| StorageError::Io(e))?;
        Ok(file)
    }

    /// Removing a saved file
    async fn delete(&self, key: PathBuf) -> Result<(), StorageError> {
        remove_file(key).await.map_err(|e| StorageError::Io(e))
    }
}
