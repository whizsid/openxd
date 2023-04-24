//! Messages that specific to application

use transport::ui::{OpenFileMessage, UIMessage};

pub enum ConnectionStartMessage {
    OpenFile(OpenFileMessage),
}

impl TryFrom<UIMessage> for ConnectionStartMessage {
    type Error = ();

    fn try_from(value: UIMessage) -> Result<Self, Self::Error> {
        match value {
            UIMessage::OpenFile(cache_id) => Ok(ConnectionStartMessage::OpenFile(
                OpenFileMessage::new(cache_id),
            )),
            _ => Err(()),
        }
    }
}
