use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UIMessage {
    Ping,
    Close,
    Error(String),
    OpenFile(String)
}

#[derive(Clone)]
pub struct PingMessage;

impl Into<UIMessage> for PingMessage {
    fn into(self) -> UIMessage {
        UIMessage::Ping
    }
}

impl TryFrom<UIMessage> for PingMessage {
    type Error = ();

    fn try_from(value: UIMessage) -> Result<PingMessage, ()> {
        match value {
            UIMessage::Ping => Ok(PingMessage),
            _=> Err(())
        }
    }
}

#[derive(Clone)]
pub struct OpenFileMessage(String);

impl OpenFileMessage {
    pub fn new(cache_id: String) -> OpenFileMessage {
        OpenFileMessage(cache_id)
    }
}

impl Into<UIMessage> for OpenFileMessage {
    fn into(self) -> UIMessage {
        UIMessage::OpenFile(self.0)
    }
}

impl TryFrom<UIMessage> for OpenFileMessage {
    type Error = ();

    fn try_from(value: UIMessage) -> Result<Self, Self::Error> {
        match value {
            UIMessage::OpenFile(cache_id) => Ok(OpenFileMessage(cache_id)),
            _=> Err(())
        }
    }
}
