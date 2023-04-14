use std::{
    pin::Pin,
    sync::mpsc::{channel, Receiver},
    task::{Context, Poll},
};

use futures::{Sink, SinkExt, Stream};

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

pub struct Client<T: Stream<Item = IncomingItem> + Sink<OutgoingItem, Error = ()> + Unpin> {
    internal: T,
    subsribers: Vec<EventSubscriber>,
}

struct EventSubscriber {
    consume: Box<dyn Fn(IncomingItem) -> bool>,
    once: bool,
}

impl EventSubscriber {
    pub fn new(consume: Box<dyn Fn(IncomingItem) -> bool>, once: bool) -> EventSubscriber {
        EventSubscriber { consume, once }
    }

    pub fn notify(&self, message: IncomingItem) -> bool {
        self.consume.as_ref()(message)
    }

    pub fn is_one_time(&self) -> bool {
        self.once
    }
}

impl<T: Stream<Item = IncomingItem> + Sink<OutgoingItem, Error = ()> + Unpin> Client<T> {
    pub fn new(internal: T) -> Client<T> {
        Client {
            internal,
            subsribers: Vec::new(),
        }
    }

    /// Send a request and waiting for a response
    pub async fn send_and_receive<
        OT: Into<OutgoingItem>,
        IT: TryFrom<IncomingItem, Error = ()> + 'static,
    >(
        &mut self,
        message: OT,
    ) -> Result<IT, ()> {
        let mut pin_outgoing = Pin::new(&mut self.internal);
        let out_message: OutgoingItem = message.into();
        pin_outgoing.send(out_message).await?;

        let (sub_id, receiver) = self.new_subscriber_internal::<IT>(true);

        let response = receiver.recv().unwrap();

        self.remove_subscriber(sub_id);

        // We are only filtering the values that matching for this type for the channel
        Ok(response)
    }

    /// Send a message without caring about a response
    pub async fn send(&mut self, message: OutgoingItem) -> Result<(), ()> {
        let mut pin_outgoing: Pin<&mut T> = Pin::new(&mut self.internal);
        pin_outgoing.send(message).await
    }

    pub fn remove_subscriber(&mut self, index: usize) {
        self.subsribers.remove(index);
    }

    pub fn new_subscriber<IT: TryFrom<IncomingItem> + 'static>(&mut self) -> (usize, Receiver<IT>) {
        self.new_subscriber_internal::<IT>(false)
    }

    fn new_subscriber_internal<'a, IT: TryFrom<IncomingItem> + 'static>(
        &mut self,
        once: bool,
    ) -> (usize, Receiver<IT>) {
        let (sender, receiver) = channel();
        let es = EventSubscriber::new(
            Box::new(move |i| {
                let converted = IT::try_from(i);
                if let Ok(incoming_message) = converted {
                    sender.send(incoming_message).is_ok()
                } else {
                    false
                }
            }),
            once,
        );
        self.subsribers.push(es);
        (self.subsribers.len() - 1, receiver)
    }

    /// Sync only if one time subscribers there
    pub fn sync_once(&mut self, cx: &mut Context) {
        let has_once = self.subsribers.iter().find(|s| s.is_one_time()).is_some();
        if has_once {
            let incoming_pin: Pin<&mut T> = Pin::new(&mut self.internal);
            if let Poll::Ready(message_opt) = incoming_pin.poll_next(cx) {
                if let Some(message) = message_opt {
                    for es in &self.subsribers {
                        if es.notify(message.clone()) {
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Synchronize the pending messages
    pub fn sync(&mut self, cx: &mut Context) {
        let incoming_pin: Pin<&mut T> = Pin::new(&mut self.internal);
        if let Poll::Ready(message_opt) = incoming_pin.poll_next(cx) {
            if let Some(message) = message_opt {
                for es in &self.subsribers {
                    if es.notify(message.clone()) {
                        break;
                    }
                }
            }
        }
    }
}
