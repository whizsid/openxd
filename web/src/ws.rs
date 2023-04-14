use std::task::Poll;

use futures::{Stream, Sink};
use pin_project::pin_project;
use ws_stream_wasm::{WsErr, WsMessage, WsMeta, WsStream};

#[pin_project]
pub struct WebSocket {
    #[pin]
    internal: WsStream,
    meta: WsMeta,
}

#[derive(Debug)]
pub struct WebSocketError(WsErr);

impl From<WsErr> for WebSocketError {
    fn from(value: WsErr) -> Self {
        WebSocketError(value)
    }
}

impl WebSocket {
    pub async fn connect(url: &str) -> Result<WebSocket, WebSocketError> {
        let (ws, wsstream) = WsMeta::connect(url, None).await?;
        Ok(WebSocket {
            meta: ws,
            internal: wsstream,
        })
    }

    pub async fn close(&self) -> Result<(), WebSocketError> {
        self.meta.close().await?;
        Ok(())
    }
}

impl Stream for WebSocket {
    type Item = Vec<u8>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        let polled: Poll<Option<WsMessage>> = this.internal.poll_next(cx);
        polled.map(|m_opt| {
            m_opt.map(|m| match m {
                WsMessage::Text(_) => Vec::new(),
                WsMessage::Binary(data) => data,
            })
        })
    }
}

impl Sink<Vec<u8>> for WebSocket {
    type Error = WebSocketError;
    
    fn start_send(self: std::pin::Pin<&mut Self>, item: Vec<u8>) -> Result<(), Self::Error> {
        let this = self.project();
        let result: Result<(), WsErr> = this.internal.start_send(WsMessage::Binary(item));
        result.map_err(|e|WebSocketError(e))
    }

    fn poll_ready(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let ready: Poll<Result<(), WsErr>> = this.internal.poll_ready(cx);
        ready.map(|r|r.map_err(|e|WebSocketError(e)))
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let flushed = this.internal.poll_flush(cx);
        flushed.map(|r|r.map_err(|e|WebSocketError(e)))
    }

    fn poll_close(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let closed = this.internal.poll_close(cx);
        closed.map(|r|r.map_err(|e|WebSocketError(e)))
    }
}
