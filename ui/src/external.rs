//! External calls to the backend

use std::sync::Arc;

use async_trait::async_trait;

#[cfg_attr(target_arch="wasm32",async_trait(?Send))]
#[cfg_attr(not(target_arch="wasm32"),async_trait)]
pub trait External: Send + Sync + 'static {
    type Error;

    /// Create a project using an existing OXD file
    async fn create_project_using_existing_file(self: Arc<Self>, buf: Vec<u8>, project_name: String) -> Result<String, Self::Error>;

    async fn save_current_snapshot(self: Arc<Self>) -> Result<(), Self::Error>;
}
