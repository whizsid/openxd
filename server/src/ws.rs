use std::task::Poll;

use futures::{Sink, Stream};
use pin_project::pin_project;
use tokio_tungstenite::WebSocketStream;
use transport::ui::UIMessage;
use tungstenite::{Error as TungsteniteError, Message};
use tokio::net::TcpStream;
use bincode::serialize as to_bin;

#[pin_project]
pub struct WebSocket(#[pin] WebSocketStream<TcpStream>);

#[derive(Debug)]
pub struct WebSocketError(TungsteniteError);

impl WebSocket {
    pub fn new(internal: WebSocketStream<TcpStream>) -> WebSocket {
        WebSocket(internal)
    }
}

impl Sink<Vec<u8>> for WebSocket {
    type Error = WebSocketError;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let polled: Poll<Result<(), TungsteniteError>> = this.0.poll_ready(cx);
        polled.map(|r|r.map_err(|e|WebSocketError(e)))
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: Vec<u8>) -> Result<(), Self::Error> {
        let this = self.project();
        let result: Result<(), TungsteniteError> = this.0.start_send(Message::Binary(item));
        result.map_err(|e|WebSocketError(e))
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let polled = this.0.poll_flush(cx);
        polled.map(|r|r.map_err(|e|WebSocketError(e)))
    }

    fn poll_close(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let polled = this.0.poll_close(cx);
        polled.map(|r|r.map_err(|e|WebSocketError(e)))
    }
}

impl Stream for WebSocket {
    type Item = Vec<u8>;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let polled: Poll<Option<Result<Message,TungsteniteError>>> = this.0.poll_next(cx);
        polled.map(|o| o.map(|r| match r {
            Ok(m)=> match m {
                Message::Binary(bin) => bin,
                Message::Close(_) => to_bin(&UIMessage::Close).unwrap(),
                _ => unreachable!()
            },
            Err(e) => to_bin(&UIMessage::Error(e.to_string())).unwrap()
        }))
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
