use std::{fmt::Debug, rc::Rc};

use poll_promise::Promise;

use crate::{
    client::ClientTransport, commands::Command, remote_cache::RemoteCache, scopes::ApplicationScope,
};

pub struct FileOpenCommand<
    TE: Debug,
    CE: Debug,
    T: ClientTransport<TE> + Send + 'static,
    C: RemoteCache<Error = CE> + Send + Sync + 'static,
> {
    app_scope: Rc<ApplicationScope<TE, CE, T, C>>,
    file_dialog_promise: Option<Promise<Option<Vec<u8>>>>,
    opened_file_cache_promise: Option<Promise<Result<String, String>>>,
    file_open_promise: Option<Promise<Result<(), ()>>>,
}

impl<
        TE: Debug,
        CE: Debug,
        T: ClientTransport<TE> + Send + 'static,
        C: RemoteCache<Error = CE> + Send + Sync + 'static,
    > FileOpenCommand<TE, CE, T, C>
{
    pub fn new(app_scope: Rc<ApplicationScope<TE, CE, T, C>>) -> Self {
        let mut state_mut = app_scope.state_mut();
        state_mut.disable_main_ui();
        state_mut.set_status_message(String::from("Opening the file"));
        drop(state_mut);

        let file_dialog_promise = Promise::spawn_async(async move {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("OpenXD", &["oxd"])
                .pick_file()
                .await;
            if let Some(f) = file {
                Some(f.read().await)
            } else {
                None
            }
        });

        FileOpenCommand {
            app_scope,
            file_dialog_promise: Some(file_dialog_promise),
            opened_file_cache_promise: None::<Promise<Result<String, String>>>,
            file_open_promise: None::<Promise<Result<(), ()>>>,
        }
    }

    pub fn file_dialog_cancel(&mut self) {
        let mut state_mut = self.app_scope.state_mut();
        state_mut.enable_main_ui();
        state_mut.clear_status_message();
        drop(state_mut);
    }

    pub fn cache_opened_file(&mut self, buf: Vec<u8>) {
        self.app_scope
            .state_mut()
            .set_status_message("Caching the opened file");
        let cache = self.app_scope.remote_cache();
        let _ = self
            .opened_file_cache_promise
            .insert(Promise::spawn_async(async move {
                let cached_res = cache.cache(buf).await;
                cached_res.map_err(|e| format!("{:?}", e))
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
        drop(state_mut);
    }

    pub fn open_file(&mut self, cache_id: String) {
        self.app_scope
            .state_mut()
            .set_status_message("Opening the file");
    }
}

impl<
        TE: Debug,
        CE: Debug,
        T: ClientTransport<TE> + Send + 'static,
        C: RemoteCache<Error = CE> + Send + Sync + 'static,
    > Command for FileOpenCommand<TE, CE, T, C>
{
    fn update(&mut self) -> bool {
        let done = false;
        if let Some(file_dialog_promise) = self.file_dialog_promise.take() {
            if let Some(buf_opt) = file_dialog_promise.ready() {
                self.file_dialog_promise = None;

                if let Some(buf) = buf_opt {
                    self.cache_opened_file(buf.clone());
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

        done
    }
}
