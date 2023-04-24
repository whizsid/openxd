use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ApplicationMessage {
    FileOpened,
    Error(String)
}

pub struct FileOpenedMessage;

impl FileOpenedMessage {
    pub fn new() -> FileOpenedMessage {
        FileOpenedMessage
    }
}

impl TryFrom<ApplicationMessage> for FileOpenedMessage {
    type Error = ();
    fn try_from(value: ApplicationMessage) -> Result<Self, Self::Error> {
        match value {
            ApplicationMessage::FileOpened=> Ok(FileOpenedMessage),
            _ => Err(())
        }
    }
}

impl Into<ApplicationMessage> for FileOpenedMessage{
    fn into(self) -> ApplicationMessage {
        ApplicationMessage::FileOpened
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
