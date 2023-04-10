use std::{
    pin::Pin,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    task::{Context, Poll},
};

use futures::{Sink, SinkExt, Stream, StreamExt};

pub use futures::sink::Send;

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

pub struct Client<I: Stream<Item = IncomingItem> + Unpin, O: Sink<OutgoingItem, Error = ()> + Unpin>
{
    incoming: Arc<Mutex<I>>,
    outgoing: O,
    subsribers: Vec<EventSubscriber>,
}

struct EventSubscriber {
    is_satisfied: Box<dyn Fn(IncomingItem) -> bool>,
    sender: Sender<IncomingItem>,
}

impl EventSubscriber {
    pub fn new(
        is_satisfied: Box<dyn Fn(IncomingItem) -> bool>,
        sender: Sender<IncomingItem>,
    ) -> EventSubscriber {
        EventSubscriber {
            is_satisfied,
            sender,
        }
    }

    pub fn satisfied(&self, message: IncomingItem) -> bool {
        self.is_satisfied.as_ref()(message)
    }

    pub fn notify(&self, message: IncomingItem) {
        self.sender.send(message).unwrap()
    }
}

impl<I: Stream<Item = IncomingItem> + Unpin, O: Sink<OutgoingItem, Error = ()> + Unpin>
    Client<I, O>
{
    pub fn new(income: I, outgo: O) -> Client<I, O> {
        Client {
            incoming: Arc::new(Mutex::new(income)),
            outgoing: outgo,
            subsribers: Vec::new(),
        }
    }

    /// Send a request and waiting for a response
    pub async fn send_and_receive<OT: Into<OutgoingItem>, IT: TryFrom<IncomingItem, Error = ()>>(
        &mut self,
        message: OT,
    ) -> Result<IT, ()> {
        let mut pin_outgoing = Pin::new(&mut self.outgoing);
        let out_message: OutgoingItem = message.into();
        pin_outgoing.send(out_message).await?;

        let (sub_id, receiver) = self.new_subscriber::<IT>();

        let response = receiver.recv().unwrap();

        self.remove_subscriber(sub_id);

        // We are only filtering the values that matching for this type for the channel
        Ok(IT::try_from(response).unwrap())
    }

    /// Send a message without caring about a response
    pub async fn send(&mut self, message: OutgoingItem) -> Result<(), ()> {
        let mut pin_outgoing = Pin::new(&mut self.outgoing);
        pin_outgoing.send(message).await
    }

    pub fn remove_subscriber(&mut self, index: usize) {
        self.subsribers.remove(index);
    }

    pub fn new_subscriber<IT: TryFrom<IncomingItem>>(&mut self) -> (usize, Receiver<IncomingItem>) {
        let (sender, receiver) = channel();
        let es = EventSubscriber::new(Box::new(|i| -> bool { IT::try_from(i).is_ok() }), sender);
        self.subsribers.push(es);
        (self.subsribers.len() - 1, receiver)
    }

    /// Synchronize the pending messages
    pub fn sync(&mut self, cx: &mut Context) {
        let mut incoming = self.incoming.lock().unwrap();
        let incoming_pin: Pin<&mut I> = Pin::new(&mut incoming);
        if let Poll::Ready(message_opt) = incoming_pin.poll_next(cx) {
            if let Some(message) = message_opt {
                for es in &self.subsribers {
                    let message = message.clone();
                    if es.satisfied(message.clone()) {
                        es.notify(message);
                    }
                }
            }
        }
    }
}
