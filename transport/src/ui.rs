use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UIMessage {
    Ping,
    Close,
    Error(String),
    OpenFile(String),
    NewProject(String),
    Resize(u32, u32),
    CloseTab(String),
}

#[derive(Clone)]
pub struct OpenFileMessage {
    pub project_id: String,
}

impl OpenFileMessage {
    pub fn new(project_id: String) -> OpenFileMessage {
        OpenFileMessage { project_id }
    }
}

impl Into<UIMessage> for OpenFileMessage {
    fn into(self) -> UIMessage {
        UIMessage::OpenFile(self.project_id)
    }
}

impl TryFrom<UIMessage> for OpenFileMessage {
    type Error = ();

    fn try_from(value: UIMessage) -> Result<Self, Self::Error> {
        match value {
            UIMessage::OpenFile(project_id) => Ok(OpenFileMessage { project_id }),
            _ => Err(()),
        }
    }
}

pub struct NewProjectMessage {
    pub project_name: String,
}

impl NewProjectMessage {
    pub fn new(project_name: String) -> NewProjectMessage {
        NewProjectMessage { project_name }
    }
}

impl Into<UIMessage> for NewProjectMessage {
    fn into(self) -> UIMessage {
        UIMessage::NewProject(self.project_name)
    }
}

impl TryFrom<UIMessage> for NewProjectMessage {
    type Error = ();

    fn try_from(value: UIMessage) -> Result<Self, Self::Error> {
        match value {
            UIMessage::NewProject(project_name) => Ok(NewProjectMessage::new(project_name)),
            _ => Err(()),
        }
    }
}

pub struct ResizeMessage {
    pub width: u32,
    pub height: u32
}

impl Into<UIMessage> for ResizeMessage {
    fn into(self) -> UIMessage {
        UIMessage::Resize(self.width, self.height)
    }
}

impl TryFrom<UIMessage> for ResizeMessage {
    type Error = ();

    fn try_from(value: UIMessage) -> Result<Self, Self::Error> {
        match value {
            UIMessage::Resize(width, height) => Ok(ResizeMessage { width, height }),
            _ => Err(())
        }
    }
}

pub struct CloseTabMessage {
    pub tab_id: String
}

impl Into<UIMessage> for CloseTabMessage {
    fn into(self) -> UIMessage {
        UIMessage::CloseTab(self.tab_id)
    }
}

impl TryFrom<UIMessage> for CloseTabMessage {
    type Error = ();

    fn try_from(value: UIMessage) -> Result<Self, Self::Error> {
        match value {
            UIMessage::CloseTab(tab_id) => Ok(CloseTabMessage { tab_id }),
            _ => Err(())
        }
    }
}
