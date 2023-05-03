use ui::cache::Cache as UICache;

pub struct UserCache;

#[derive(Debug)]
pub enum UserCacheError {
    PermissionDenied,
    IoError,
}

impl UserCache {
    pub fn new() -> UserCache {
        UserCache
    }
}

impl UICache for UserCache {
    type Error = UserCacheError;

    fn cache_file<'async_trait>(self:std::sync::Arc<Self> ,buf:Vec<u8>) ->  core::pin::Pin<Box<dyn core::future::Future<Output = Result<String,Self::Error> > + core::marker::Send+'async_trait> >where Self:'async_trait {
        let fut = async {
            Ok(String::new())
        };
        Box::pin(fut)
    }
}
