use std::fmt::Debug;

use futures::{Sink, Stream};

pub fn draw_menu_bar<E: Debug, T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin + Send +'static>(
    egui: &mut egui::Ui,
    app: &mut crate::ui::Ui<E, T>,
) {
    egui::menu::bar(egui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Open").clicked() {
                ui.close_menu();
                app.open_file_dialog();
            }
        });
    });
}
