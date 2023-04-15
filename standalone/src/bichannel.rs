use futures::{
    self,
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender, SendError},
    Sink, Stream
};
use pin_project::pin_project;
use std::task::Poll;

#[derive(Debug)]
pub struct BiChannelError(SendError);

#[pin_project]
pub struct BiChannel<I, O> {
    #[pin]
    receiver: UnboundedReceiver<I>,
    #[pin]
    sender: UnboundedSender<O>,
}

impl<I, O> BiChannel<I, O> {
    pub fn new<T, Y>() -> (BiChannel<T, Y>, BiChannel<Y, T>) {
        let (sender1, receiver1) = unbounded();
        let (sender2, receiver2) = unbounded();

        (
            BiChannel {
                receiver: receiver2,
                sender: sender1,
            },
            BiChannel {
                receiver: receiver1,
                sender: sender2,
            },
        )
    }
}

impl<I, O> Stream for BiChannel<I, O> {
    type Item = I;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        this.receiver.poll_next(cx)
    }
}

impl<I, O> Sink<O> for BiChannel<I, O> {
    type Error = BiChannelError;

    fn start_send(self: std::pin::Pin<&mut Self>, item: O) -> Result<(), Self::Error> {
        let this = self.project();
        let result: Result<(), SendError> = this.sender.start_send(item);
        result.map_err(|e|BiChannelError(e))
    }

    fn poll_ready(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let polled: Poll<Result<(), SendError>> = this.sender.poll_ready(cx);
        polled.map(|r|r.map_err(|e| BiChannelError(e)))
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let polled: Poll<Result<(), SendError>> = this.sender.poll_flush(cx);
        polled.map(|r|r.map_err(|e| BiChannelError(e)))
    }

    fn poll_close(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        let polled: Poll<Result<(), SendError>> = this.sender.poll_close(cx);
        polled.map(|r|r.map_err(|e| BiChannelError(e)))
    }
}

