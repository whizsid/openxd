//! External calls to the backend

use std::sync::Arc;

use async_trait::async_trait;

#[async_trait]
pub trait External: Send + Sync + 'static {
    type Error;

    /// Create a project using an existing OXD file
    async fn create_project_using_existing_file(self: Arc<Self>, buf: Vec<u8>, project_name: String) -> Result<String, Self::Error>;
}
