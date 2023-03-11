use eframe::egui::{menu, CentralPanel, Context, TopBottomPanel};
use poll_promise::Promise;

use crate::{app::App, components::menu::draw_menu_bar};

pub struct Ui {
    app: App,
    file_open_promise: Option<Promise<Option<Vec<u8>>>>,
}

impl Ui {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            app: App::new(),
            file_open_promise: None::<Promise<Option<Vec<u8>>>>,
        }
    }
}

impl Ui {
    pub(crate) fn open_file_dialog(&mut self) {
        self.app.file_dialog_opened();
        let _ = self.file_open_promise
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
}

impl eframe::App for Ui {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
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
