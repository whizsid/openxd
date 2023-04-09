use std::{sync::mpsc::{channel, Receiver, Sender}, pin::Pin};

use futures::{Sink, Stream, SinkExt, Future};
use incoming::IncomingItem;
use outgoing::{OutgoingError, OutgoingItem};
use pin_project::pin_project;

pub use futures::sink::Send;

pub mod incoming;
pub mod outgoing;

#[pin_project]
pub struct Client<
    I: Stream<Item = incoming::IncomingItem>,
    O: Sink<outgoing::OutgoingItem, Error = OutgoingError>,
> {
    incoming: I,
    #[pin]
    outgoing: O,
    subsribers: Vec<EventSubscriber>,
}

struct EventSubscriber {
    is_satisfied: Box<dyn Fn(&incoming::IncomingItem) -> bool>,
    sender: Sender<incoming::IncomingItem>,
}

impl EventSubscriber {
    pub fn new(
        is_satisfied: Box<dyn Fn(&IncomingItem) -> bool>,
        sender: Sender<incoming::IncomingItem>,
    ) -> EventSubscriber {
        EventSubscriber {
            is_satisfied,
            sender,
        }
    }
}

impl<
        I: Stream<Item = incoming::IncomingItem>,
        O: Sink<outgoing::OutgoingItem, Error = OutgoingError>,
    > Client<I, O>
{
    pub fn new(income: I, outgo: O) -> Client<I, O> {
        Client {
            incoming: income,
            outgoing: outgo,
            subsribers: Vec::new(),
        }
    }

    pub fn send_and_receive(self: Pin<&mut Self>, message: OutgoingItem) -> Result<IncomingItem, OutgoingError> {
        let this = self.project();
        match this.outgoing.start_send(message) {
            Ok(_) => {

                

                Err(OutgoingError)
            },
            Err(e) => {return Err(e); }
        }
    }

    pub async fn send(self: Pin<&mut Self>, message: OutgoingItem) -> Result<(), OutgoingError>  {
        let mut this = self.project();

        this.outgoing.send(message).await
    }

    pub fn remove_subscriber(&mut self, index: usize) {
        self.subsribers.remove(index);
    }

    pub fn new_subscriber(
        &mut self,
        is_satisfied: Box<dyn Fn(&IncomingItem) -> bool>,
    ) -> (usize, Receiver<incoming::IncomingItem>) {
        let (sender, receiver) = channel();
        let es = EventSubscriber::new(Box::new(is_satisfied), sender);
        self.subsribers.push(es);
        (self.subsribers.len() - 1, receiver)
    }
}

pub struct SendAndReceive {

}
