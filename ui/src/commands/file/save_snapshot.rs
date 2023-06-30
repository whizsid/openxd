use poll_promise::Promise;

use crate::{
    commands::Command, scopes::ApplicationScope,
};

pub struct SaveSnapshotCommand {
    app_scope: ApplicationScope,
    snapshot_save_promise: Promise<Result<(), String>>,
}

impl SaveSnapshotCommand {
    pub fn new(app_scope: ApplicationScope) -> SaveSnapshotCommand {
        let mut state_mut = app_scope.state_mut();
        state_mut.disable_main_ui();
        state_mut.set_status_message("Saving");
        drop(state_mut);
        let external_client = app_scope.external_client();
        SaveSnapshotCommand {
            app_scope,
            snapshot_save_promise: Promise::spawn_async(async move {
                let res = external_client.save_current_snapshot().await;
                res.map_err(|e| format!("{:?}", e))
            }),
        }
    }

    pub fn snapshot_saved(&mut self) {
        let mut state = self.app_scope.state_mut();
        state.clear_status_message();
        state.enable_main_ui();
    }

    pub fn snapshot_save_failed(&mut self, err: String) {
        let mut state = self.app_scope.state_mut();
        state.clear_status_message();
        state.enable_main_ui();
        state.add_dialog(
            crate::state::Severity::Error,
            format!("Failed to save the project. Original Error:- {}", err),
        );
    }
}

impl Command for SaveSnapshotCommand {
    fn update(&mut self) -> bool {
        if let Some(save_res) = self.snapshot_save_promise.ready() {
            match save_res {
                Ok(_) => {
                    self.snapshot_saved();
                }
                Err(e) => {
                    self.snapshot_save_failed(e.clone());
                }
            }
            true
        } else {
            false
        }
    }
}
