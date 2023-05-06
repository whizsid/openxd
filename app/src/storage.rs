//! Storages can be vary on the environment and platform
//!
//! We can use S3 as production storage, File system as the staging storage,
//! user's file system as the desktop storage.

use std::{fmt::Debug, hash::Hash};

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use tokio::io::{AsyncRead, AsyncWrite};

pub trait StorageId: Hash + PartialEq + Eq + Serialize + DeserializeOwned + Sync + Send {}

impl <T> StorageId for T where T:Hash + PartialEq + Eq + Serialize + DeserializeOwned + Sync + Send {}

/// Storage interface to interact with file system
#[async_trait]
pub trait Storage<E: Debug, ID: StorageId> {
    /// Saving a file to storage
    ///
    /// Namespace is used to identify the type of the file
    async fn put<'a, I: AsyncRead>(&self, file: &'a mut I, namespace: String) -> Result<ID, E>;

    /// Retrieving the saved file from the storage
    async fn get<O: AsyncWrite>(&self, key: ID) -> Result<O, E>;

    /// Removing a saved file
    async fn delete(&self, key: ID) -> Result<(), E>;
}
