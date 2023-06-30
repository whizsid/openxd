use poll_promise::Promise;

use crate::{
    commands::Command, scopes::ApplicationScope,
};

pub struct TabCloseCommand {
    app_scope: ApplicationScope,
    tab_idx: usize,
    close_tab_promise: Promise<Result<(), String>>,
}

impl TabCloseCommand {
    pub fn new(app_scope: ApplicationScope, tab_idx: usize) -> TabCloseCommand {
        let tab = app_scope.state().tab(tab_idx).unwrap();
        let tab_borrowed = tab.borrow();
        let tab_id = tab_borrowed.id();

        app_scope.state_mut().set_status_message("Closing");

        let client = app_scope.client();
        let close_tab_promise = Promise::spawn_async(async move {
            let mut client = client.lock().await;
            client
                .close_tab(tab_id)
                .await
                .map_err(|e| format!("{:?}", e))
        });
        TabCloseCommand {
            app_scope,
            tab_idx,
            close_tab_promise,
        }
    }

    pub fn tab_close_failed(&mut self, err_msg: String) {
        let tab = self.app_scope.state().tab(self.tab_idx).unwrap();
        let mut tab_borrowed = tab.borrow_mut();
        tab_borrowed.set_closing(false);

        let mut state_mut = self.app_scope.state_mut();
        state_mut.add_dialog(
            crate::state::Severity::Error,
            format!(
                "Error occured during closing the tab. Original error:- {}",
                err_msg
            ),
        );
        state_mut.clear_status_message();
    }

    pub fn tab_close_success(&mut self) {
        self.app_scope.remove_tab(self.tab_idx);
        self.app_scope.state_mut().clear_status_message();
    }
}

impl Command for TabCloseCommand {
    fn update(&mut self) -> bool {
        if let Some(res) = self.close_tab_promise.ready() {
            if let Err(msg) = res {
                self.tab_close_failed(msg.clone());
            } else {
                self.tab_close_success();
            }
            true
        } else {
            false
        }
    }
}
