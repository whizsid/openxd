use bincode::{deserialize as from_bin, serialize as to_bin, ErrorKind as BincodeError};
use futures::{Sink, SinkExt, Stream, StreamExt};
use std::{pin::Pin, fmt::Debug};

pub mod app;
pub mod ui;

#[cfg(feature = "ui")]
use app::ApplicationMessage as IncomingItem;
#[cfg(feature = "ui")]
use ui::UIMessage as OutgoingItem;

#[cfg(feature = "app")]
use app::ApplicationMessage as OutgoingItem;
#[cfg(feature = "app")]
use ui::UIMessage as IncomingItem;

pub struct Client<E: Debug, T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin> {
    internal: T,
    pending: Vec<IncomingItem>,
}

#[derive(Debug)]
pub enum SendError<E: Debug> {
    Serialize(BincodeError),
    Send(E),
}

#[derive(Debug)]
pub enum ReceiveError {
    Deserialize(BincodeError),
}

#[derive(Debug)]
pub enum SendAndReceiveError<E: Debug> {
    Send(SendError<E>),
    Receive(ReceiveError),
}

impl <E:Debug> From<SendError<E>> for SendAndReceiveError<E> {
    fn from(value: SendError<E>) -> Self {
        SendAndReceiveError::Send(value)
    }
}

impl <E:Debug> From<ReceiveError> for SendAndReceiveError<E> {
    fn from(value: ReceiveError) -> Self {
        SendAndReceiveError::Receive(value)
    }
}

impl<E: Debug, T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin> Client<E, T> {
    pub fn new(internal: T) -> Client<E, T> {
        Client {
            internal,
            pending: Vec::new(),
        }
    }

    /// Send a request and waiting for a response
    pub async fn send_and_receive<OT: Into<OutgoingItem>, IT: TryFrom<IncomingItem, Error = ()>>(
        &mut self,
        message: OT,
    ) -> Result<IT, SendAndReceiveError<E>> {
        self.send(message).await?;
        let response = self.receive::<IT>().await?;
        Ok(response)
    }

    pub async fn receive<IT: TryFrom<IncomingItem, Error = ()>>(
        &mut self,
    ) -> Result<IT, ReceiveError> {
        for (i, pending_message) in self.pending.iter().enumerate() {
            if let Ok(converted_message) = IT::try_from(pending_message.clone()) {
                self.pending.remove(i);
                return Ok(converted_message);
            }
        }

        loop {
            let response_message_opt = self.internal.next().await;
            if let Some(bin_message) = response_message_opt {
                match from_bin::<IncomingItem>(&bin_message) {
                    Ok(response_message) => {
                        if let Ok(converted_message) = IT::try_from(response_message.clone()) {
                            return Ok(converted_message);
                        } else {
                            self.pending.push(response_message);
                        }
                    }
                    Err(e) => {
                        return Err(ReceiveError::Deserialize(*e));
                    }
                }
            }
        }
    }

    /// Send a message without caring about a response
    pub async fn send<OT: Into<OutgoingItem>>(&mut self, message: OT) -> Result<(), SendError<E>> {
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
