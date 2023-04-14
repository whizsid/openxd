use std::fmt::Debug;

use futures::{Sink, Stream};
use transport::app::PongMessage;
use transport::ui::PingMessage;
use transport::Client as InternalClient;

pub struct Client<E: Debug, T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin> {
    internal: InternalClient<E, T>,
}

impl<E:Debug, T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin> Client<E,T> {
    pub fn new(internal: T) -> Client<E, T> {
        Client {
            internal: InternalClient::new(internal),
        }
    }

    pub async fn ping(&mut self) -> Result<(), ()> {
        self.internal.send_and_receive::<PingMessage, PongMessage>(PingMessage).await.unwrap();
        Ok(())
    }
}
