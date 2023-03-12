use async_trait::async_trait;
use tarpc::service;

#[service]
#[async_trait]
pub trait OpenXD {
    async fn open_local() -> Result<String, String>;
}
