use std::{cell::RefCell, rc::Rc};

use crate::state::AppState;

pub struct App {
    state: AppState,
}

impl App {
    pub fn new() -> Self {
        Self { state: AppState::new() }
    }

    pub fn file_dialog_opened(&mut self) {
        self.state.disable_main_ui();
    }

    pub fn file_dialog_done(&mut self, _buf: Vec<u8>) {
        self.state.enable_main_ui();
    }

    pub fn file_dilaog_canceled(&mut self) {
        self.state.enable_main_ui();
    }
    
    pub fn state(&self) -> &AppState{
        &self.state
    }
}
