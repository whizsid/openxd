use futures::{Sink, Stream};
use std::fmt::Debug;
use transport::{app::{ApplicationMessage, PongMessage}, ui::{UIMessage, PingMessage}, Client};

pub struct App {}

impl App {
    pub fn new() -> App {
        App {}
    }

    pub fn create_session<
        E: Debug,
        T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin,
    >(
        &self,
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
        self.handle_ping().await;
    }

    pub async fn handle_ping(&mut self) {
        self.client.receive::<PingMessage>().await.expect("Can not get ping message");
        println!("Ping Message Received");
        self.client.send(PongMessage).await.expect("Can not send pong message");
    }
}
