use futures::{Sink, Stream};
use messages::ConnectionStartMessage;
use std::fmt::Debug;
use transport::{app::{ApplicationMessage, ErrorMessage, FileOpenedMessage}, ui::UIMessage, Client};

mod messages;

pub struct App {}

impl App {
    pub fn new() -> App {
        App {}
    }

    pub fn create_session<
        E: Debug,
        T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin,
    >(
        &mut self,
        internal_client: T,
    ) -> Session<E, T> {
        Session::new(internal_client)
    }
}

pub struct Session<E: Debug, T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin> {
    client: Client<UIMessage, ApplicationMessage, E, T>,
}

impl<E: Debug, T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin> Session<E, T> {
    pub fn new(internal_client: T) -> Session<E, T> {
        Session {
            client: Client::new(internal_client),
        }
    }

    pub async fn start(&mut self) {
        let start_message_res = self.client.receive::<ConnectionStartMessage>().await;
        match start_message_res {
            Ok(start_message) => {
                self.client.send(FileOpenedMessage::new()).await.unwrap();
            },
            Err(start_err) => {
                self.client.send(ErrorMessage::new(format!("{:?}", start_err))).await.unwrap();
            }
        }
    }
}
