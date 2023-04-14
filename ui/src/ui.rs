use std::fmt::Debug;

use egui::{CentralPanel, Context, TopBottomPanel};
use futures::{Stream, Sink};
use poll_promise::Promise;

use crate::{app::App, components::menu::draw_menu_bar};

pub struct Ui<E: Debug, T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin> {
    app: App<E, T>,
    file_open_promise: Option<Promise<Option<Vec<u8>>>>,
}

impl <E: Debug, T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin> Ui<E, T> {
    pub fn new(transport: T) -> Self {
        Self {
            app: App::new(transport),
            file_open_promise: None::<Promise<Option<Vec<u8>>>>,
        }
    }

    pub(crate) fn open_file_dialog(&mut self) {
        self.app.file_dialog_opened();
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

    // Updatng the UI in one iteration in the event loop
    pub fn update(&mut self, ctx: &Context) {
        if let Some(file_open_promise) = self.file_open_promise.take() {
            if let Some(file_content_opt) = file_open_promise.ready() {
                self.file_open_promise = None;
                if let Some(buf) = file_content_opt {
                    self.app.file_dialog_done(buf.to_vec());
                } else {
                    self.app.file_dilaog_canceled();
                }
            } else {
                self.file_open_promise.replace(file_open_promise);
            }
        }

        TopBottomPanel::top("menu-bar").show(ctx, |ui| {
            ui.add_enabled_ui(!self.app.state().is_main_ui_disabled(), |ui| {
                draw_menu_bar(ui, self);
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!self.app.state().is_main_ui_disabled(), |ui| {
                ui.heading("Hello World!");
            })
        });
    }
}
