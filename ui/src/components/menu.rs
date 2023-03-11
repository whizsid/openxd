pub fn draw_menu_bar(egui: &mut eframe::egui::Ui, app: &mut crate::ui::Ui) {
    eframe::egui::menu::bar(egui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Open").clicked() {
                ui.close_menu();
                app.open_file_dialog();
            }
        });
    });
}
