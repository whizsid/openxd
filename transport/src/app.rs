use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ApplicationMessage {
    Error(String),
    TabCreated(TabCreatedMessage),
    Pong
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TabCreatedMessage {
    pub tab_name: String,
    pub tab_id: String,
}

impl TabCreatedMessage {
    pub fn new(tab_name: String, tab_id: String) -> TabCreatedMessage {
        TabCreatedMessage { tab_name, tab_id }
    }
}

impl TryFrom<ApplicationMessage> for TabCreatedMessage {
    type Error = ();

    fn try_from(value: ApplicationMessage) -> Result<Self, Self::Error> {
        match value {
            ApplicationMessage::TabCreated(inner) => Ok(inner),
            _ => Err(())
        }
    }
}

impl Into<ApplicationMessage> for TabCreatedMessage {
    fn into(self) -> ApplicationMessage {
        ApplicationMessage::TabCreated(self)
    }
}

pub struct ErrorMessage(String);

impl ErrorMessage {
    pub fn new(err: String) -> ErrorMessage {
        ErrorMessage(err)
    }
}

impl TryFrom<ApplicationMessage> for ErrorMessage {
    type Error = ();
    fn try_from(value: ApplicationMessage) -> Result<Self, Self::Error> {
        match value {
            ApplicationMessage::Error(err)=> Ok(ErrorMessage::new(err)),
            _ => Err(())
        }
    }
}

impl Into<ApplicationMessage> for ErrorMessage {
    fn into(self) -> ApplicationMessage {
        ApplicationMessage::Error(self.0)
    }
}

pub struct PongMessage;

impl PongMessage {
    pub fn new() -> PongMessage {
        PongMessage
    }
}

impl TryFrom<ApplicationMessage> for PongMessage {
    type Error = ();
    fn try_from(value: ApplicationMessage) -> Result<Self, Self::Error> {
        match value {
            ApplicationMessage::Pong => Ok(PongMessage::new()),
            _ => Err(())
        }
    }
}

impl Into<ApplicationMessage> for PongMessage {
    fn into(self) -> ApplicationMessage {
        ApplicationMessage::Pong
    }
}
