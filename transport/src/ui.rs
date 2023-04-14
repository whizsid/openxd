use serde::{Serialize, Deserialize};

#[derive(Clone)]
pub struct PingMessage;

impl Into<UIMessage> for PingMessage {
    fn into(self) -> UIMessage {
        UIMessage::Ping
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum UIMessage {
    Test1,
    Test2,
    Ping,
}
