use std::fmt::Debug;
use std::marker::PhantomData;

use futures::{Sink, Stream};
use transport::app::{ApplicationMessage, PongMessage};
use transport::ui::{PingMessage, UIMessage};
use transport::Client as InternalClient;

pub trait ClientTransport<E: Debug>: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin {}

impl <E:Debug, T>ClientTransport<E> for T where T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin {}

pub struct Client<E: Debug, T: ClientTransport<E>> {
    internal: InternalClient<ApplicationMessage, UIMessage, E, T>,
    _phantom: PhantomData<E>
}

impl<E: Debug, T: ClientTransport<E>> Client<E, T> {
    pub fn new(internal: T) -> Client<E, T> {
        Client {
            internal: InternalClient::new(internal),
            _phantom: PhantomData
        }
    }

    pub async fn ping(&mut self) -> Result<(), ()> {
        self.internal
            .send_and_receive::<PingMessage, PongMessage>(PingMessage)
            .await
            .unwrap();
        Ok(())
    }
}
