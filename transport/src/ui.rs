use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UIMessage {
    Ping,
    Close,
    Error(String),
    OpenFile(String),
    NewFile,
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

pub struct NewFileMessage;

impl NewFileMessage {
    pub fn new() -> NewFileMessage {
        NewFileMessage
    }
}

impl Into<UIMessage> for NewFileMessage {
    fn into(self) -> UIMessage {
        UIMessage::NewFile
    }
}

impl TryFrom<UIMessage> for NewFileMessage {
    type Error = ();

    fn try_from(value: UIMessage) -> Result<Self, Self::Error> {
        match value {
            UIMessage::NewFile => Ok(NewFileMessage),
            _ => Err(())
        }
    }
}
