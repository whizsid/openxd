use std::{cell::RefCell, fmt::Debug, rc::Rc, sync::Arc};

use egui::Ui;
use futures::{Sink, Stream, lock::Mutex};
use poll_promise::Promise;

use crate::{client::Client, remote_cache::RemoteCache, state::AppState};

pub struct FileMenuComponent<
    TE: Debug + 'static,
    CE: Debug,
    T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = TE> + Unpin + Send + 'static,
    C: RemoteCache<Error = CE> + Send + Sync + 'static,
> {
    app_state: Rc<RefCell<AppState>>,
    client: Arc<Mutex<Client<TE, T>>>,
    remote_cache: Arc<C>,

    // Promises
    file_open_promise: Option<Promise<Option<Vec<u8>>>>,
    opened_file_cache_promise: Option<Promise<Result<String, String>>>,
}

impl<
        TE: Debug + 'static,
        CE: Debug,
        T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = TE> + Unpin + Send + 'static,
        C: RemoteCache<Error = CE> + Send + Sync + 'static,
    > FileMenuComponent<TE, CE, T, C>
{
    pub fn new(
        app_state: Rc<RefCell<AppState>>,
        client: Arc<Mutex<Client<TE, T>>>,
        remote_cache: Arc<C>,
    ) -> Self {
        FileMenuComponent {
            app_state,
            client,
            remote_cache,
            file_open_promise: None::<Promise<Option<Vec<u8>>>>,
            opened_file_cache_promise: None::<Promise<Result<String, String>>>,
        }
    }

    pub(crate) fn open_file_dialog(&mut self) {
        self.app_state.borrow_mut().disable_main_ui();
        let _ = self
            .file_open_promise
            .insert(Promise::spawn_async(async move {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("OpenXD", &["oxd"])
                    .pick_file()
                    .await;
                if let Some(f) = file {
                    Some(f.read().await)
                } else {
                    None
                }
            }));
    }

    pub fn cache_opened_file(&mut self, buf: Vec<u8>) {
        let cache = self.remote_cache.clone();
        let _ = self.opened_file_cache_promise.insert(Promise::spawn_async(async move {
            let cached_res = cache.cache(buf).await;
            cached_res.map_err(|e| format!("{:?}", e))
        }));
    }

    pub fn update(&mut self, ui: &mut Ui) {
        if let Some(file_open_promise) = self.file_open_promise.take() {
            if let Some(file_content_opt) = file_open_promise.ready() {
                self.file_open_promise = None;
                if let Some(buf) = file_content_opt {
                    self.cache_opened_file(buf.clone());
                } else {
                    self.app_state.borrow_mut().enable_main_ui();
                }
            } else {
                self.file_open_promise.replace(file_open_promise);
            }
        }

        if let Some(opened_file_cache_promise) = self.opened_file_cache_promise.take() {
            if let Some(file_cached) = opened_file_cache_promise.ready() {
                
            } else {
                self.opened_file_cache_promise.replace(opened_file_cache_promise);
            }
        }

        ui.menu_button("File", |ui| {
            if ui.button("Open").clicked() {
                ui.close_menu();
                self.open_file_dialog();
            }
        });
    }
}
