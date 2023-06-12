use std::fmt::Debug;
use std::marker::PhantomData;

use futures::{Sink, Stream};
use transport::app::{ApplicationMessage, TabCreatedMessage};
use transport::ui::{NewProjectMessage, OpenFileMessage, UIMessage};
use transport::{Client as InternalClient, SendAndReceiveError};

/// Trait constraints to internal transport of the `Client`
pub trait ClientTransport<E: Debug + Send>:
    Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin + Send + 'static
{
}

impl<E: Debug + Send, T> ClientTransport<E> for T where
    T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin + Send + 'static
{
}

/// Response type that expect either error or a success response
pub struct ResultResponse<T: TryFrom<ApplicationMessage, Error = ()>>(Result<T, String>);

impl<T: TryFrom<ApplicationMessage, Error = ()>> ResultResponse<T> {
    pub fn error(err: String) -> Self {
        ResultResponse(Err(err))
    }

    pub fn ok(ok: T) -> Self {
        ResultResponse(Ok(ok))
    }
}

impl<T: TryFrom<ApplicationMessage, Error = ()>> TryFrom<ApplicationMessage> for ResultResponse<T> {
    type Error = ();
    fn try_from(value: ApplicationMessage) -> Result<Self, Self::Error> {
        match value {
            ApplicationMessage::Error(err) => Ok(ResultResponse::error(err)),
            _ => T::try_from(value).map(|r| ResultResponse::ok(r)),
        }
    }
}

impl<T: TryFrom<ApplicationMessage, Error = ()>> Into<Result<T, String>> for ResultResponse<T> {
    fn into(self) -> Result<T, String> {
        self.0
    }
}

/// Main transport media between UI and application logics
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

    /// Opening a cached file
    pub async fn file_open(
        &mut self,
        project_id: String,
    ) -> Result<ResultResponse<TabCreatedMessage>, SendAndReceiveError<E>> {
        self.internal
            .send_and_receive::<OpenFileMessage, ResultResponse<TabCreatedMessage>>(
                OpenFileMessage::new(project_id),
            )
            .await
    }

    /// Creating a new project
    pub async fn create_new_project(
        &mut self,
        project_name: String,
    ) -> Result<ResultResponse<TabCreatedMessage>, SendAndReceiveError<E>> {
        self.internal
            .send_and_receive::<NewProjectMessage, ResultResponse<TabCreatedMessage>>(
                NewProjectMessage::new(project_name),
            )
            .await
    }
}
