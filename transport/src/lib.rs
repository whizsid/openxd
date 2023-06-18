use bincode::{deserialize as from_bin, serialize as to_bin, ErrorKind as BincodeError};
use futures::{Sink, SinkExt, Stream, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, marker::PhantomData, pin::Pin};

pub mod app;
pub mod ui;
pub mod vo;

#[derive(Debug)]
pub enum SendError<E: Debug> {
    Serialize(BincodeError),
    Send(E),
}

#[derive(Debug)]
pub enum ReceiveError {
    Deserialize(BincodeError),
    Terminated,
}

#[derive(Debug)]
pub enum SendAndReceiveError<E: Debug> {
    Send(SendError<E>),
    Receive(ReceiveError),
}

impl<E: Debug> From<SendError<E>> for SendAndReceiveError<E> {
    fn from(value: SendError<E>) -> Self {
        SendAndReceiveError::Send(value)
    }
}

impl<E: Debug> From<ReceiveError> for SendAndReceiveError<E> {
    fn from(value: ReceiveError) -> Self {
        SendAndReceiveError::Receive(value)
    }
}

pub struct Client<
    I: Serialize + DeserializeOwned + Clone + Debug + Sized,
    O: Serialize + DeserializeOwned + Clone + Debug + Sized,
    E: Debug,
    T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin,
> {
    internal: T,
    pending: Vec<I>,
    terminated: bool,
    _out: PhantomData<O>,
}

impl<
        I: Serialize + DeserializeOwned + Clone + Debug + Sized,
        O: Serialize + DeserializeOwned + Clone + Debug + Sized,
        E: Debug,
        T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin,
    > Client<I, O, E, T>
{
    pub fn new(internal: T) -> Client<I, O, E, T> {
        Client {
            internal,
            pending: Vec::new(),
            _out: PhantomData,
            terminated: false,
        }
    }

    /// Send a request and waiting for a response
    pub async fn send_and_receive<OT: Into<O>, IT: TryFrom<I, Error = ()>>(
        &mut self,
        message: OT,
    ) -> Result<IT, SendAndReceiveError<E>> {
        self.send(message).await?;
        let response = self.receive::<IT>().await?;
        Ok(response)
    }

    /// Receive any incoming message without converting or blocking
    pub async fn receive_raw(&mut self) -> Result<I, ReceiveError> {
        let response_message_opt = self.internal.next().await;
        if let Some(bin_message) = response_message_opt {
            match from_bin::<I>(&bin_message) {
                Ok(response_message) => Ok(response_message),
                Err(e) => Err(ReceiveError::Deserialize(*e)),
            }
        } else {
            self.terminated = true;
            Err(ReceiveError::Terminated)
        }
    }

    pub async fn receive<IT: TryFrom<I, Error = ()>>(&mut self) -> Result<IT, ReceiveError> {
        for (i, pending_message) in self.pending.iter().enumerate() {
            if let Ok(converted_message) = IT::try_from(pending_message.clone()) {
                self.pending.remove(i);
                return Ok(converted_message);
            }
        }

        if self.terminated {
            return Err(ReceiveError::Terminated);
        }

        loop {
            let response_message = self.receive_raw().await?;

            if let Ok(converted_message) = IT::try_from(response_message.clone()) {
                return Ok(converted_message);
            } else {
                self.pending.push(response_message);
            }
        }
    }

    /// Send a message without caring about a response
    pub async fn send<OT: Into<O>>(&mut self, message: OT) -> Result<(), SendError<E>> {
        let mut pin_internal: Pin<&mut T> = Pin::new(&mut self.internal);
        match to_bin(&message.into()) {
            Ok(serialized) => pin_internal
                .send(serialized)
                .await
                .map_err(|e| SendError::Send(e)),
            Err(e) => Err(SendError::Serialize(*e)),
        }
    }
}
