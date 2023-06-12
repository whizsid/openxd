use std::{fmt::Debug, rc::Rc};

use poll_promise::Promise;

use crate::{external::External, client::ClientTransport, scopes::ApplicationScope, commands::Command};

pub struct SaveSnapshotCommand<
    TE: Debug + Send + 'static,
    EE: Debug + 'static,
    T: ClientTransport<TE>,
    E: External<Error = EE>,
> {
    app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
    snapshot_save_promise: Promise<Result<(), String>>,
}

impl<TE: Debug + Send + 'static, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>>
    SaveSnapshotCommand<TE, EE, T, E>
{
    pub fn new(
        app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
    ) -> SaveSnapshotCommand<TE, EE, T, E> {
        let external_client = app_scope.external_client();
        let mut state_mut = app_scope.state_mut();
        state_mut.disable_main_ui();
        state_mut.set_status_message("Saving");
        drop(state_mut);
        SaveSnapshotCommand { 
            app_scope,
            snapshot_save_promise: Promise::spawn_async(async {
                let res = external_client.save_current_snapshot().await;
                res.map_err(|e|format!("{:?}", e))
            })
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
        state.add_dialog(crate::state::Severity::Error, format!("Failed to save the project. Original Error:- {}", err));
    }
}

impl<TE: Debug + Send + 'static, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>> Command
    for SaveSnapshotCommand<TE, EE, T, E>
{
    fn update(&mut self) -> bool {
        if let Some(save_res) = self.snapshot_save_promise.ready() {
            match save_res {
                Ok(_) => {
                    self.snapshot_saved();
                },
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
