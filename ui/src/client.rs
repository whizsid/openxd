use std::fmt::Debug;
use std::marker::PhantomData;

use futures::{Sink, Stream};
use transport::app::{ApplicationMessage, FileOpenedMessage, PongMessage};
use transport::ui::{OpenFileMessage, PingMessage, UIMessage};
use transport::{Client as InternalClient, SendAndReceiveError};

pub trait ClientTransport<E: Debug + Send>:
    Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin + Send + 'static
{
}

impl<E: Debug + Send, T> ClientTransport<E> for T where
    T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin + Send + 'static
{
}

pub struct Client<E: Debug + Send, T: ClientTransport<E>> {
    internal: InternalClient<ApplicationMessage, UIMessage, E, T>,
    _phantom: PhantomData<E>,
}

impl<E: Debug + Send, T: ClientTransport<E>> Client<E, T> {
    pub fn new(internal: T) -> Client<E, T> {
        Client {
            internal: InternalClient::new(internal),
            _phantom: PhantomData,
        }
    }

    pub async fn ping(&mut self) -> Result<(), ()> {
        self.internal
            .send_and_receive::<PingMessage, PongMessage>(PingMessage)
            .await
            .unwrap();
        Ok(())
    }

    pub async fn file_open(&mut self, cache_id: String) -> Result<(), SendAndReceiveError<E>> {
        self.internal
            .send_and_receive::<OpenFileMessage, FileOpenedMessage>(OpenFileMessage::new(cache_id))
            .await
            .map(|_| ())
    }
}
