use serde::{Serialize, Deserialize};

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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UIMessage {
    Test1,
    Test2,
    Ping,
    Close,
    Error(String)
}
