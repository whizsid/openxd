use std::path::PathBuf;

use app::storage::{Storage, StorageObjInfo};
use async_trait::async_trait;
use tokio::{
    fs::{remove_file, File},
    io::{copy, AsyncRead},
};
use uuid::Uuid;

use crate::config::STORAGE_FS_ROOT;

pub struct FileSystemStorage;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Failed to read or write some data")]
    Io(#[from] std::io::Error),
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
            .await?;
        copy(reader, &mut file)
            .await?;
        Ok(path.to_path_buf())
    }

    /// Retrieving the saved file from the storage
    async fn get(&self, key: PathBuf) -> Result<Self::Read, StorageError> {
        let file = File::open(key).await?;
        Ok(file)
    }

    /// Removing a saved file
    async fn delete(&self, key: PathBuf) -> Result<(), StorageError> {
        remove_file(key).await.map_err(|e| StorageError::Io(e))
    }

    /// Retrieve the information of a storage object
    async fn info(&self, key: PathBuf) -> Result<StorageObjInfo, StorageError> {
        let mut ext_opt: Option<String> = None;
        if let Some(ext) = key.extension() {
            if let Some(ext_str) = ext.to_str() {
                ext_opt = Some(String::from(ext_str));
            }
        }
        Ok(StorageObjInfo {
            ext: ext_opt
        })
    }

    /// Duplicating a storage object
    async fn duplicate(&self, key: PathBuf) -> Result<PathBuf, StorageError> {
        let ext = key.extension();
        let file_name = if let Some(ext) = ext {
            if let Some(ext) = ext.to_str() {
                format!("{}.{}", Uuid::new_v4(), ext)
            } else {
                Uuid::new_v4().to_string()
            }
        } else {
            Uuid::new_v4().to_string()
        };
        let mut new_path = key.clone();
        new_path.set_file_name(file_name);

        tokio::fs::copy(key, new_path.clone()).await?;

        Ok(new_path)
    }
}
