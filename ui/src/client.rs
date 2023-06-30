use std::fmt::Debug;
use std::marker::PhantomData;

use async_trait::async_trait;
use futures::{Sink, Stream};
use transport::app::{ApplicationMessage, TabCreatedMessage};
use transport::ui::{CloseTabMessage, NewProjectMessage, OpenFileMessage, UIMessage};
use transport::{
    Client as InternalClient, SendAndReceiveError as InternalSendAndReceiveError,
    SendError as InternalSendError,
};

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

#[derive(Debug)]
pub enum SendAndReceiveError {
    ResponseError(String),
    SendAndReceiveError(InternalSendAndReceiveError<String>),
}

#[derive(Debug)]
pub struct SendError(InternalSendError<String>);

impl<E: Debug> From<InternalSendError<E>> for SendError {
    fn from(value: InternalSendError<E>) -> Self {
        match value {
            InternalSendError::Serialize(ser) => {
                SendError(InternalSendError::Serialize(ser))
            }
            InternalSendError::Send(e) => {
                SendError(InternalSendError::Send(format!("{:?}", e)))
            }
        }
    }
}

fn convert_res<R: TryFrom<ApplicationMessage, Error = ()>, E: Debug>(
    err: Result<ResultResponse<R>, InternalSendAndReceiveError<E>>,
) -> Result<R, SendAndReceiveError> {
    match err {
        Ok(res) => match res.0 {
            Ok(sr) => Ok(sr),
            Err(e) => Err(SendAndReceiveError::ResponseError(e)),
        },
        Err(e) => match e {
            InternalSendAndReceiveError::Send(se) => match se {
                InternalSendError::Send(sei) => {
                    Err(SendAndReceiveError::SendAndReceiveError(
                        InternalSendAndReceiveError::Send(InternalSendError::Send(format!("{:?}", sei))),
                    ))
                }
                InternalSendError::Serialize(ser) => {
                    Err(SendAndReceiveError::SendAndReceiveError(
                        InternalSendAndReceiveError::Send(InternalSendError::Serialize(ser)),
                    ))
                }
            },
            InternalSendAndReceiveError::Receive(re) => {
                Err(SendAndReceiveError::SendAndReceiveError(
                    InternalSendAndReceiveError::Receive(re),
                ))
            }
        },
    }
}

#[async_trait]
pub trait Client: Send + 'static {
    async fn file_open(
        &mut self,
        project_id: String,
    ) -> Result<TabCreatedMessage, SendAndReceiveError>;

    async fn create_new_project(
        &mut self,
        project_name: String,
    ) -> Result<TabCreatedMessage, SendAndReceiveError>;

    async fn close_tab(&mut self, tab_id: String) -> Result<(), SendError>;
}

#[async_trait]
impl<E: Debug + Send + 'static, T: ClientTransport<E>> Client for ClientImpl<E, T> {
    async fn file_open(
        &mut self,
        project_id: String,
    ) -> Result<TabCreatedMessage, SendAndReceiveError> {
        convert_res(self._file_open(project_id).await)
    }

    async fn create_new_project(
        &mut self,
        project_name: String,
    ) -> Result<TabCreatedMessage, SendAndReceiveError> {
        convert_res(self._create_new_project(project_name).await)
    }

    async fn close_tab(&mut self, tab_id: String) -> Result<(), SendError> {
        self._close_tab(tab_id).await.map_err(|e|e.into())
    }
}

/// Main transport media between UI and application logics
pub struct ClientImpl<E: Debug + Send, T: ClientTransport<E>> {
    internal: InternalClient<ApplicationMessage, UIMessage, E, T>,
    _phantom: PhantomData<E>,
}

impl<E: Debug + Send, T: ClientTransport<E>> ClientImpl<E, T> {
    pub fn new(internal: T) -> ClientImpl<E, T> {
        ClientImpl {
            internal: InternalClient::new(internal),
            _phantom: PhantomData,
        }
    }

    /// Opening a cached file
    async fn _file_open(
        &mut self,
        project_id: String,
    ) -> Result<ResultResponse<TabCreatedMessage>, InternalSendAndReceiveError<E>> {
        self.internal
            .send_and_receive::<OpenFileMessage, ResultResponse<TabCreatedMessage>>(
                OpenFileMessage::new(project_id),
            )
            .await
    }

    /// Creating a new project
    async fn _create_new_project(
        &mut self,
        project_name: String,
    ) -> Result<ResultResponse<TabCreatedMessage>, InternalSendAndReceiveError<E>> {
        self.internal
            .send_and_receive::<NewProjectMessage, ResultResponse<TabCreatedMessage>>(
                NewProjectMessage::new(project_name),
            )
            .await
    }

    async fn _close_tab(&mut self, tab_id: String) -> Result<(), InternalSendError<E>> {
        self.internal.send(CloseTabMessage::new(tab_id)).await
    }
}
