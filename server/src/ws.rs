use std::task::Poll;

use bincode::serialize as to_bin;
use futures::{Sink, Stream};
use pin_project::pin_project;
use routerify_websocket::{
    Message, WebSocket as RouterifyWebSocket, WebsocketError as RouterifyError,
};
use transport::ui::UIMessage;

#[pin_project]
pub struct WebSocket(#[pin] RouterifyWebSocket);

#[derive(Debug)]
pub struct WebSocketError(RouterifyError);

impl WebSocket {
    pub fn new(internal: RouterifyWebSocket) -> WebSocket {
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
        let polled: Poll<Result<(), RouterifyError>> = this.0.poll_ready(cx);
        polled.map(|r| r.map_err(|e| WebSocketError(e)))
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: Vec<u8>) -> Result<(), Self::Error> {
        let this = self.project();
        let result: Result<(), RouterifyError> = this.0.start_send(Message::binary(item));
        result.map_err(|e| WebSocketError(e))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let polled = this.0.poll_flush(cx);
        polled.map(|r| r.map_err(|e| WebSocketError(e)))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let polled = this.0.poll_close(cx);
        polled.map(|r| r.map_err(|e| WebSocketError(e)))
    }
}

impl Stream for WebSocket {
    type Item = Vec<u8>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let polled: Poll<Option<Result<Message, RouterifyError>>> = this.0.poll_next(cx);
        polled.map(|o| {
            o.map(|r| match r {
                Ok(m) => {
                    if m.is_close() {
                        to_bin(&UIMessage::Close).unwrap()
                    } else if m.is_binary() {
                        m.into_bytes()
                    } else {
                        unreachable!()
                    }
                }
                Err(e) => to_bin(&UIMessage::Error(e.to_string())).unwrap(),
            })
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
