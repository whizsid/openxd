pub struct AppState {
    main_ui_disabled: bool
}

impl AppState {
    pub fn new() -> AppState {
        AppState{
            main_ui_disabled: false,
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
}
