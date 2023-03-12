use tarpc::context;
use rpc::OpenXD;

#[derive(Clone)]
pub struct OpenXDServer {}

#[tarpc::server]
#[async_trait::async_trait]
impl OpenXD for OpenXDServer {
    async fn open_local(self, _: context::Context) -> Result<String, String> {
        Ok(String::from("File opened"))
    }
}
