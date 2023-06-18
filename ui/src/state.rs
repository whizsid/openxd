//! UI States
use std::{slice::Iter, rc::Rc, cell::RefCell};

use transport::vo::Screen;

use crate::{commands::Command, tab::TabInfo};

/// Application wide states
pub struct AppState {
    /// Whether the main UI is disabled or not
    main_ui_disabled: bool,
    /// List of dialogs with user. Such as errors, info boxes
    dialogs: Vec<AppDialog>,
    /// Used to generate unique ids to dialogs
    dialog_counter: usize,
    /// Message to display in status bar
    status_message: Option<String>,
    /// Whether that new project dialog opened or not
    new_project_opened: bool,
    opened_projects: Vec<Rc<RefCell<TabInfo>>>,
}

impl AppState {
    /// Creating a new state
    pub fn new() -> AppState {
        AppState {
            main_ui_disabled: false,
            dialogs: vec![],
            dialog_counter: 0,
            status_message: None,
            new_project_opened: false,
            opened_projects: vec![]
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
    pub fn add_dialog<T: Into<String>>(
        &mut self,
        severity: Severity,
        message: T,
    ) -> &mut AppDialog {
        let err_len = self.dialogs.len();
        self.dialogs.push(AppDialog::new(
            self.dialog_counter,
            severity,
            message.into(),
        ));
        self.dialog_counter += 1;
        self.dialogs.get_mut(err_len).unwrap()
    }

    /// Removing the dialog by id
    ///
    /// This will return the removed `AppDialog` instance if found any for the provided unique id
    /// fetched from the `AppDialog::id` method. You can call the close
    /// callback command using the returned `AppDialog`
    pub fn remove_dialog(&mut self, index: usize) -> Option<AppDialog> {
        let dialog_index_opt = self.dialogs.iter().position(|d| d.id() == index);
        if let Some(dialog_index) = dialog_index_opt {
            return Some(self.dialogs.remove(dialog_index));
        }
        None
    }

    /// Get a reference to a single dialog by the id
    pub fn get_dialog(&self, index: usize) -> Option<&AppDialog> {
        self.dialogs.iter().find(|d| d.id() == index)
    }

    /// Dialog count
    pub fn len_dialogs(&self) -> usize {
        self.dialogs.len()
    }

    /// Iterator to cloned dialogs
    ///
    /// These dialog instances will not contain the command initiators. Use the a reference using
    /// the `get_dialog` or `remove_dialog` to access the command initiator
    pub fn dialogs(&self) -> Vec<AppDialog> {
        self.dialogs.clone()
    }

    /// Opening the new project window
    pub fn open_new_project_dialog(&mut self) {
        self.new_project_opened = true;
        self.main_ui_disabled = true;
    }

    /// Closing the new project window
    pub fn close_new_project_dialog(&mut self) {
        self.new_project_opened = false;
        self.main_ui_disabled = false;
    }

    /// Whether that new project window opened or not
    pub fn is_new_project_dialog_opened(&self) -> bool {
        self.new_project_opened
    }

    /// Adding a project as a tab
    pub fn add_project(&mut self, id: String, title: String, zoom: f64, screens: Vec<Screen>) {
        self.opened_projects.push(Rc::new(RefCell::new(TabInfo::new(id, title, zoom, screens))));
    }

    /// Retrieving a tab by index
    pub fn tab(&self, index: usize) -> Option<Rc<RefCell<TabInfo>>> {
        self.opened_projects.get(index).map(|t|t.clone()).clone()
    }

    /// Returning the opened projects count
    pub fn tab_count(&self) -> usize {
        self.opened_projects.len()
    }
}

/// Severity of dialogs and dialog buttons
#[derive(Clone)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Success,
}

/// A button that should display on a button
pub struct AppDialogButton {
    /// Severity of the button. The button color, font color is changing according to
    /// this severity
    severity: Severity,
    /// Text to display on the button
    text: String,
    /// This callback will create a new command which execute on click events.
    command: Option<Box<dyn FnOnce() -> Box<dyn Command + 'static>>>,
    /// Unique id of the button in app dialog
    id: usize,
}

impl AppDialogButton {
    pub fn new(id: usize, severity: Severity, text: String) -> AppDialogButton {
        AppDialogButton {
            id,
            severity,
            text,
            command: None,
        }
    }

    /// Setting a callback to initiate the command which need to trigger on click events.
    pub fn on_click<F>(&mut self, cmd_init: F)
    where
        F: FnOnce() -> Box<dyn Command + 'static> + 'static,
    {
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

    /// Returning the unique id for the button
    pub fn id(&self) -> usize {
        self.id
    }

    /// Creating a command using the command initiator and returning it.
    pub fn create_command(self) -> Option<Box<dyn Command + 'static>> {
        if let Some(cmd_init) = self.command {
            let cmd = cmd_init();
            Some(cmd)
        } else {
            None
        }
    }
}

impl Clone for AppDialogButton {
    /// Cloning the button partially. This clone will droping the callback function.
    /// Fetch the original reference from the app state if you need to access the callback.
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            severity: self.severity.clone(),
            text: self.text.clone(),
            command: None,
        }
    }
}

pub struct AppDialog {
    severity: Severity,
    message: String,
    buttons: Vec<AppDialogButton>,
    id: usize,
    on_close_cmd: Option<Box<dyn FnOnce() -> Box<dyn Command + 'static>>>,
}

impl AppDialog {
    pub fn new(id: usize, severity: Severity, message: String) -> AppDialog {
        AppDialog {
            severity,
            message,
            buttons: vec![],
            id,
            on_close_cmd: None,
        }
    }

    /// Adding a button to the dialog
    ///
    /// You can set callbacks using the returned `&mut AppDialogButton`
    pub fn add_button(&mut self, severity: Severity, text: String) -> &mut AppDialogButton {
        let len_btn = self.buttons.len();
        self.buttons
            .push(AppDialogButton::new(len_btn, severity, text));
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

    /// Returning the unique id for the dialog
    pub fn id(&self) -> usize {
        self.id
    }

    /// Popping out a button from the dialog.
    pub fn pop_button(&mut self, index: usize) -> Option<AppDialogButton> {
        if let Some(_) = self.buttons.get(index) {
            Some(self.buttons.remove(index))
        } else {
            None
        }
    }

    /// Getting a single button by id
    pub fn button(&self, index: usize) -> Option<&AppDialogButton> {
        self.buttons.get(index)
    }

    /// Setting a callback to initiate the command which need to trigger on close events.
    pub fn on_close<F>(&mut self, cmd_init: F)
    where
        F: FnOnce() -> Box<dyn Command + 'static> + 'static,
    {
        self.on_close_cmd = Some(Box::new(cmd_init));
    }

    /// Creating the on_close command using the command initiator and returning it.
    pub fn create_close_command(self) -> Option<Box<dyn Command + 'static>> {
        if let Some(cmd_init) = self.on_close_cmd {
            let cmd = cmd_init();
            Some(cmd)
        } else {
            None
        }
    }

    pub fn iter_buttons(&self) -> Iter<'_, AppDialogButton> {
        self.buttons.iter()
    }
}

impl Clone for AppDialog {
    /// Partially cloning the application details. The on_close command executor will be not
    /// included the cloned instance. Use the original reference from the app_state if you need to
    /// access the command initiator.
    fn clone(&self) -> Self {
        Self {
            severity: self.severity.clone(),
            message: self.message.clone(),
            buttons: self.buttons.clone(),
            id: self.id,
            on_close_cmd: None,
        }
    }
}

/// States related to create a project window
pub struct CreateProjectWindowState {
    project_name: String
}

impl CreateProjectWindowState {
    pub fn new() -> CreateProjectWindowState {
        CreateProjectWindowState { project_name: String::new() }
    }

    pub fn change_project_name(&mut self, name: String) {
        self.project_name = name;
    }

    pub fn get_project_name(&self) -> String {
        self.project_name.clone()
    }
}
