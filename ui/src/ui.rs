use std::{cell::RefCell, fmt::Debug, rc::Rc, sync::Arc};

use egui::{CentralPanel, Context, TopBottomPanel};
use futures::{Sink, Stream};
use futures::lock::Mutex;
use log::{error, info};
use poll_promise::Promise;

use crate::{client::Client, components::menu::draw_menu_bar, state::AppState};

pub struct Ui<
    E: Debug + 'static,
    T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin + Send + 'static,
> {
    state: AppState,
    client: Arc<Mutex<Client<E, T>>>,
    file_open_promise: Option<Promise<Option<Vec<u8>>>>,
    ping_promise: Option<Promise<Result<(), ()>>>,
}

impl<
        E: Debug + 'static,
        T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin + Send + 'static,
    > Ui<E, T>
{
    pub fn new(transport: T) -> Self {
        Self {
            file_open_promise: None::<Promise<Option<Vec<u8>>>>,
            client: Arc::new(Mutex::new(Client::new(transport))),
            state: AppState::new(),
            ping_promise: None::<Promise<Result<(), ()>>>,
        }
    }

    pub(crate) fn open_file_dialog(&mut self) {
        self.state.disable_main_ui();
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

    pub(crate) fn ping(&mut self) {
        self.state.disable_main_ui();
        let client_cloned = self.client.clone();
        self.ping_promise.insert(Promise::spawn_async(async move {
            let mut client = client_cloned.lock().await;
            client.ping().await;
            Ok(())
        }));
    }

    // Updatng the UI in one iteration in the event loop
    pub fn update(&mut self, ctx: &Context) {
        if let Some(file_open_promise) = self.file_open_promise.take() {
            if let Some(file_content_opt) = file_open_promise.ready() {
                self.file_open_promise = None;
                if let Some(buf) = file_content_opt {
                    // Do anything with buffer
                    self.state.enable_main_ui();
                } else {
                    self.state.enable_main_ui();
                    self.ping();
                }
            } else {
                self.file_open_promise.replace(file_open_promise);
            }
        }

        if let Some(ping_promise) = self.ping_promise.take() {
            if let Some(ping_res) = ping_promise.ready() {
                self.ping_promise = None;
                self.state.enable_main_ui();
                if let Ok(_) = ping_res {
                    info!("Pong received");
                } else {
                    error!("Pong not received");
                }
            } else {
                self.ping_promise.replace(ping_promise);
            }
        }

        TopBottomPanel::top("menu-bar").show(ctx, |ui| {
            ui.add_enabled_ui(!self.state.is_main_ui_disabled(), |ui| {
                draw_menu_bar(ui, self);
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!self.state.is_main_ui_disabled(), |ui| {
                ui.heading("Hello World!");
            })
        });
    }
}
