use crate::commands::Command;

pub struct AppState {
    main_ui_disabled: bool,
    dialogs: Vec<AppDialog>,
    status_message: Option<String>
}

impl AppState {
    pub fn new() -> AppState {
        AppState{
            main_ui_disabled: false,
            dialogs: vec![],
            status_message: None,
        }
    }

    pub fn disable_main_ui(&mut self) {
        self.main_ui_disabled = true;
    }

    pub fn enable_main_ui(&mut self) {
        self.main_ui_disabled = false;
    }

    pub fn is_main_ui_disabled(&self) -> bool {
        self.main_ui_disabled
    }

    pub fn set_status_message<T: Into<String>>(&mut self, message: T) {
        self.status_message = Some(message.into());
    }

    pub fn status_message(&self) -> Option<String> {
        self.status_message.clone()
    }

    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    pub fn add_dialog<T: Into<String>>(&mut self, severity: Severity, message: T) -> &mut AppDialog {
        let err_len = self.dialogs.len();
        self.dialogs.push(AppDialog::new(severity, message.into()));
        self.dialogs.get_mut(err_len).unwrap()
    }

    pub fn remove_dialog(&mut self, index: usize) -> AppDialog {
        self.dialogs.remove(index)
    }
}

#[derive(Clone)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Success
}

pub struct AppDialogButton {
    severity: Severity,
    text: String,
    command: Option<Box<dyn FnOnce() -> Box<dyn Command+ 'static>>>
}

impl AppDialogButton {
    pub fn new(severity: Severity, text: String) -> AppDialogButton {
        AppDialogButton { severity, text, command: None }
    }

    pub fn on_click<F>(&mut self, cmd_init: F) where F: FnOnce()-> Box<dyn Command + 'static> +'static {
        self.command = Some(Box::new(cmd_init));
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub fn severity(&self) -> Severity {
        self.severity.clone()
    }

    pub fn create_command(self) ->Option<Box<dyn Command + 'static>> {
        if let Some(cmd_init) = self.command {
            let cmd = cmd_init();
            Some(cmd)
        } else {
            None
        }
    }
}

pub struct AppDialog {
    severity: Severity,
    message: String,
    buttons: Vec<AppDialogButton>
}

impl AppDialog {
    pub fn new(severity: Severity, message: String) -> AppDialog {
        AppDialog { severity, message, buttons: vec![] }
    }

    pub fn add_button(&mut self, severity: Severity, text: String) -> &mut AppDialogButton {
        let len_btn = self.buttons.len();
        self.buttons.push(AppDialogButton::new(severity, text));
        self.buttons.get_mut(len_btn).unwrap()
    }

    pub fn severity(&self) -> Severity {
        self.severity.clone()
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }
}
