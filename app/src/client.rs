use std::{fmt::Debug, marker::PhantomData};

use futures::{Stream, Sink};
use transport::{ui::UIMessage, app::{ApplicationMessage, ErrorMessage, FileOpenedMessage}, Client as InternalClient, ReceiveError, SendError};

use crate::messages::ConnectionStartMessage;

/// Trait constraints to internal transport of the `Client`
pub trait ClientTransport<E: Debug + Send>:
    Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin + Send + 'static
{
}

impl<E: Debug + Send, T> ClientTransport<E> for T where
    T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin + Send + 'static
{
}

/// Main transport media between UI and application logics
pub struct Client<E: Debug + Send, T: ClientTransport<E>> {
    internal: InternalClient<UIMessage, ApplicationMessage, E, T>,
    _phantom: PhantomData<E>,
}

impl<E: Debug + Send, T: ClientTransport<E>> Client<E, T> {
    pub fn new(internal: T) -> Client<E, T> {
        Client {
            internal: InternalClient::new(internal),
            _phantom: PhantomData,
        }
    }

    pub async fn wait_till_init(&mut self) -> Result<ConnectionStartMessage, ReceiveError> {
        self.internal.receive::<ConnectionStartMessage>().await
    }

    pub async fn error<NE: Debug>(&mut self, err: NE)  -> Result<(), SendError<E>> {
        self.internal.send(ErrorMessage::new(format!("{:?}", err))).await
    }

    pub async fn file_opened(&mut self) -> Result<(), SendError<E>> {
        self.internal.send(FileOpenedMessage::new()).await
    }
}
