use std::{fmt::Debug, rc::Rc};

use poll_promise::Promise;
use transport::app::TabCreatedMessage;

use crate::{
    client::ClientTransport, commands::Command, external::External, scopes::ApplicationScope,
};

pub struct FileOpenCommand<
    TE: Debug + Send + 'static,
    EE: Debug + 'static,
    T: ClientTransport<TE>,
    E: External<Error = EE>,
> {
    app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
    file_dialog_promise: Option<Promise<Option<(Vec<u8>, String)>>>,
    opened_file_cache_promise: Option<Promise<Result<String, String>>>,
    file_open_promise: Option<Promise<Result<TabCreatedMessage, String>>>,
}

impl<
        TE: Debug + Send + 'static,
        EE: Debug + 'static,
        T: ClientTransport<TE>,
        E: External<Error = EE>,
    > FileOpenCommand<TE, EE, T, E>
{
    pub fn new(app_scope: Rc<ApplicationScope<TE, EE, T, E>>) -> Self {
        let mut state_mut = app_scope.state_mut();
        state_mut.disable_main_ui();
        state_mut.set_status_message(String::from("Select the file"));
        drop(state_mut);

        let file_dialog_promise = Promise::spawn_async(async move {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("OpenXD", &["oxd"])
                .pick_file()
                .await;
            if let Some(f) = file {
                Some((f.read().await, f.file_name()))
            } else {
                None
            }
        });

        FileOpenCommand {
            app_scope,
            file_dialog_promise: Some(file_dialog_promise),
            opened_file_cache_promise: None::<Promise<Result<String, String>>>,
            file_open_promise: None::<Promise<Result<TabCreatedMessage, String>>>,
        }
    }

    pub fn file_dialog_cancel(&mut self) {
        let mut state_mut = self.app_scope.state_mut();
        state_mut.enable_main_ui();
        state_mut.clear_status_message();
        drop(state_mut);
    }

    pub fn cache_opened_file(&mut self, buf: Vec<u8>, file_name: String) {
        let mut file_name_splitted = file_name.split('.');
        let file_name = file_name_splitted.next().unwrap();
        let file_name = String::from(file_name);
        self.app_scope
            .state_mut()
            .set_status_message("Caching the opened file");
        let external_client = self.app_scope.external_client();
        let _ = self
            .opened_file_cache_promise
            .insert(Promise::spawn_async(async move {
                let external_res = external_client.create_project_using_existing_file(buf, file_name).await;
                external_res.map_err(|e| format!("{:?}", e))
            }));
    }

    pub fn caching_failed(&mut self, err: String) {
        let mut state_mut = self.app_scope.state_mut();
        state_mut.add_dialog(
            crate::state::Severity::Error,
            format!(
                "Error occured during caching the opened file. Original error:- {}",
                err
            ),
        );
        state_mut.enable_main_ui();
        state_mut.clear_status_message();
    }

    pub fn open_file(&mut self, cache_id: String) {
        self.app_scope
            .state_mut()
            .set_status_message("Opening the file");
        let client = self.app_scope.client();
        let _ = self
            .file_open_promise
            .insert(Promise::spawn_async(async move {
                let mut client_locked = client.lock().await;
                let res = client_locked.file_open(cache_id).await;
                match res {
                    Err(e) => Err(format!("{:?}", e)),
                    Ok(res) => match res.into() {
                        Ok(tab_created_message) => {
                            Ok(tab_created_message)
                        },
                        Err(server_err) => Err(format!("Remote: {:?}", server_err)),
                    },
                }
            }));
    }

    pub fn file_opened(&mut self, tab_created_message: TabCreatedMessage) {
        let mut state_mut = self.app_scope.state_mut();
        state_mut.enable_main_ui();
        state_mut.clear_status_message();
    }

    pub fn file_open_failed(&mut self, err: String) {
        let mut state_mut = self.app_scope.state_mut();
        state_mut.add_dialog(
            crate::state::Severity::Error,
            format!(
                "Error occured during opening the project. Original error:- {}",
                err
            ),
        );
        state_mut.enable_main_ui();
        state_mut.clear_status_message();
    }
}

impl<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>> Command
    for FileOpenCommand<TE, EE, T, E>
{
    fn update(&mut self) -> bool {
        let mut done = false;
        if let Some(file_dialog_promise) = self.file_dialog_promise.take() {
            if let Some(buf_opt) = file_dialog_promise.ready() {
                self.file_dialog_promise = None;

                if let Some((buf, file_name)) = buf_opt {
                    self.cache_opened_file(buf.clone(), file_name.clone());
                } else {
                    self.file_dialog_cancel();
                }
            } else {
                self.file_dialog_promise.replace(file_dialog_promise);
            }
        }

        if let Some(opened_file_cache_promise) = self.opened_file_cache_promise.take() {
            if let Some(file_cached) = opened_file_cache_promise.ready() {
                self.opened_file_cache_promise = None;

                match file_cached {
                    Ok(cache_id) => {
                        self.open_file(cache_id.clone());
                    }
                    Err(cache_err) => {
                        self.caching_failed(cache_err.clone());
                    }
                }
            } else {
                self.opened_file_cache_promise
                    .replace(opened_file_cache_promise);
            }
        }

        if let Some(file_open_promise) = self.file_open_promise.take() {
            if let Some(file_opened_res) = file_open_promise.ready() {
                self.file_open_promise = None;

                match file_opened_res {
                    Ok(tab_created_message) => {
                        self.file_opened(tab_created_message.clone());
                        done = true;
                    }
                    Err(file_open_err) => {
                        self.file_open_failed(file_open_err.clone());
                    }
                }
            } else {
                self.file_open_promise.replace(file_open_promise);
            }
        }

        done
    }
}
