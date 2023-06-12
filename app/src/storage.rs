//! Storages can be vary on the environment and platform
//!
//! We can use S3 as production storage, File system as the staging storage,
//! user's file system as the desktop storage.

use std::{fmt::Debug, hash::Hash};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::AsyncRead;
use std::error::Error as StdError;

pub trait StorageId: Hash + PartialEq + Eq + Serialize + DeserializeOwned + Sync + Send + Clone {}

impl<T> StorageId for T where T: Hash + PartialEq + Eq + Serialize + DeserializeOwned + Sync + Send + Clone {}

pub struct StorageObjInfo {
    /// Extension of the file object
    pub ext: Option<String>
}

/// Storage interface to interact with file system
#[async_trait]
pub trait Storage<E: Debug + StdError, ID: StorageId> {
    type Read: AsyncRead + Unpin;
    /// Saving a file to storage
    ///
    /// Namespace is used to identify the type of the file
    async fn put<'a, I: AsyncRead + Unpin + Send>(
        &self,
        file: &'a mut I,
        namespace: String,
        ext: String,
    ) -> Result<ID, E>;

    /// Retrieving the saved file from the storage
    async fn get(&self, key: ID) -> Result<Self::Read, E>;

    /// Removing a saved file
    async fn delete(&self, key: ID) -> Result<(), E>;

    /// Retrieve the information of a storage object
    async fn info(&self, key: ID) -> Result<StorageObjInfo, E>;

    /// Duplicating a storage object
    async fn duplicate(&self, key: ID) -> Result<ID, E>;
}
