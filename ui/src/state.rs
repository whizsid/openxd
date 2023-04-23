//! UI States
use crate::commands::Command;

/// Application wide states
pub struct AppState {
    /// Whether the main UI is disabled or not
    main_ui_disabled: bool,
    /// List of dialogs with user. Such as errors, info boxes
    dialogs: Vec<AppDialog>,
    /// Message to display in status bar
    status_message: Option<String>
}

impl AppState {
    /// Creating a new state
    pub fn new() -> AppState {
        AppState{
            main_ui_disabled: false,
            dialogs: vec![],
            status_message: None,
        }
    }

    /// Disabling the whole UI
    ///
    /// users will not able to do anything after disabled the UI.
    pub fn disable_main_ui(&mut self) {
        self.main_ui_disabled = true;
    }

    /// Enabling the UI again
    ///
    /// Enabling the UI again after disabled by `disable_main_ui`
    pub fn enable_main_ui(&mut self) {
        self.main_ui_disabled = false;
    }

    /// Checking the whether that UI was disabled or not
    pub fn is_main_ui_disabled(&self) -> bool {
        self.main_ui_disabled
    }

    /// Setting the message to display on status bar
    pub fn set_status_message<T: Into<String>>(&mut self, message: T) {
        self.status_message = Some(message.into());
    }

    /// Getter for the message in status bar
    pub fn status_message(&self) -> Option<String> {
        self.status_message.clone()
    }

    /// Clearing the status bar message
    ///
    /// Clearing the message displaying in status bar
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    /// Adding a dialog to user
    ///
    /// This method will return a `&mut AppDialog` to add the buttons to dialog
    pub fn add_dialog<T: Into<String>>(&mut self, severity: Severity, message: T) -> &mut AppDialog {
        let err_len = self.dialogs.len();
        self.dialogs.push(AppDialog::new(severity, message.into()));
        self.dialogs.get_mut(err_len).unwrap()
    }

    /// Removing the dialog by id
    ///
    /// This will return the `AppDialog` instance. You can call the close callback command using
    /// the returned `AppDialog`
    pub fn remove_dialog(&mut self, index: usize) -> AppDialog {
        self.dialogs.remove(index)
    }
}

/// Severity of dialogs and dialog buttons
#[derive(Clone)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Success
}

/// A button that should display on a button
pub struct AppDialogButton {
    /// Severity of the button. The button color, font color is changing according to
    /// this severity
    severity: Severity,
    /// Text to display on the button
    text: String,
    /// This callback will create a new command which execute on click events.
    command: Option<Box<dyn FnOnce() -> Box<dyn Command+ 'static>>>
}

impl AppDialogButton {
    pub fn new(severity: Severity, text: String) -> AppDialogButton {
        AppDialogButton { severity, text, command: None }
    }

    /// Setting a callback to initiate the command which need to trigger on click events.
    pub fn on_click<F>(&mut self, cmd_init: F) where F: FnOnce()-> Box<dyn Command + 'static> +'static {
        self.command = Some(Box::new(cmd_init));
    }

    /// Getter for the text to display on button
    pub fn text(&self) -> String {
        self.text.clone()
    }

    /// Getter for the severity of the button
    pub fn severity(&self) -> Severity {
        self.severity.clone()
    }

    /// Creating a command using the command initiator and returning it.
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

    /// Adding a button to the dialog
    ///
    /// You can set callbacks using the returned `&mut AppDialogButton`
    pub fn add_button(&mut self, severity: Severity, text: String) -> &mut AppDialogButton {
        let len_btn = self.buttons.len();
        self.buttons.push(AppDialogButton::new(severity, text));
        self.buttons.get_mut(len_btn).unwrap()
    }

    /// Getter for the severity of the dialog
    pub fn severity(&self) -> Severity {
        self.severity.clone()
    }

    /// Getter for the message on the dialog
    pub fn message(&self) -> String {
        self.message.clone()
    }
}
