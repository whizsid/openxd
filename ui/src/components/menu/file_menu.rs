use egui::Ui;

use crate::{
    commands::file::{open_file::FileOpenCommand, save_snapshot::SaveSnapshotCommand},
    components::UIComponent,
    scopes::ApplicationScope,
};

pub enum FileMenuComponentEvent {
    OpenFileClicked,
    NewProjectClicked,
    SaveClicked,
}

pub struct FileMenuComponent {
    app_scope: ApplicationScope,
}

impl FileMenuComponent {
    pub fn new(app_scope: ApplicationScope) -> Self {
        FileMenuComponent { app_scope }
    }

    pub fn on(&mut self, event: FileMenuComponentEvent) {
        match event {
            FileMenuComponentEvent::OpenFileClicked => {
                self.app_scope.execute(FileOpenCommand::new(self.app_scope.clone()));
            }
            FileMenuComponentEvent::NewProjectClicked => {
                self.app_scope.state_mut().create_project_window_mut().open();
            }
            FileMenuComponentEvent::SaveClicked => {
                self.app_scope.execute(SaveSnapshotCommand::new(self.app_scope.clone()));
            }
        }
    }
}

impl UIComponent for FileMenuComponent {
    fn draw(&mut self, ui: &mut Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("Open").clicked() {
                self.on(FileMenuComponentEvent::OpenFileClicked);
                ui.close_menu();
            }
            if ui.button("New").clicked() {
                self.on(FileMenuComponentEvent::NewProjectClicked);
                ui.close_menu();
            }
            if ui.button("Save").clicked() {
                self.on(FileMenuComponentEvent::SaveClicked);
                ui.close_menu();
            }
        });
    }
}
