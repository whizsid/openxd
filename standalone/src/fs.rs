use std::path::PathBuf;

use app::storage::{Storage, StorageObjInfo};
use async_trait::async_trait;
use tokio::{
    fs::{remove_file, File},
    io::{copy, AsyncRead},
};
use uuid::Uuid;

pub struct FileSystemStorage {
    data_dir: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("failed to read/write some data")]
    Io(#[from] std::io::Error),
}

impl FileSystemStorage {
    pub fn new(data_dir: PathBuf) -> FileSystemStorage {
        FileSystemStorage { data_dir }
    }
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
            .join(self.data_dir.clone())
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

    /// Retrieve the information of a storage object
    async fn info(&self, key: PathBuf) -> Result<StorageObjInfo, StorageError> {

        let metadata = key.metadata()?;
        let size = metadata.len();
        if let Some(ext_os_str) = key.extension() {
            if let Some(ext_str) = ext_os_str.to_str() {
                return Ok(StorageObjInfo { ext: Some(String::from(ext_str)), size });
            }
        }

        Ok(StorageObjInfo { ext: None, size })
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
