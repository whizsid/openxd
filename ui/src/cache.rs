use std::sync::Arc;

use async_trait::async_trait;

#[async_trait]
pub trait Cache: Send + Sync + 'static {
    type Error;

    async fn cache_file(self: Arc<Self>, buf: Vec<u8>) -> Result<String, Self::Error>;
}
