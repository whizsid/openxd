use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ApplicationMessage {
    Test1,
    Test2,
    Pong
}

pub struct PongMessage;

impl TryFrom<ApplicationMessage> for PongMessage {
    type Error = ();
    fn try_from(value: ApplicationMessage) -> Result<Self, Self::Error> {
        match value {
            ApplicationMessage::Pong => Ok(PongMessage),
            _ => Err(())
        }
    }
}

impl Into<ApplicationMessage> for PongMessage {
    fn into(self) -> ApplicationMessage {
        ApplicationMessage::Pong
    }
}
