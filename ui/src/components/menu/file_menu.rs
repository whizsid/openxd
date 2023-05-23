use std::{fmt::Debug, rc::Rc};

use egui::Ui;

use crate::{
    client::ClientTransport, commands::file::open_file::FileOpenCommand, components::UIComponent,
    external::External, scopes::ApplicationScope,
};

pub enum FileMenuComponentEvent {
    OpenFileClicked,
}

pub struct FileMenuComponent<
    TE: Debug + Send + 'static,
    EE: Debug + 'static,
    T: ClientTransport<TE>,
    E: External<Error = EE>,
> {
    app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
}

impl<
        TE: Debug + Send + 'static,
        EE: Debug + 'static,
        T: ClientTransport<TE>,
        E: External<Error = EE>,
    > FileMenuComponent<TE, EE, T, E>
{
    pub fn new(app_scope: Rc<ApplicationScope<TE, EE, T, E>>) -> Self {
        FileMenuComponent { app_scope }
    }

    pub fn on(&mut self, event: FileMenuComponentEvent) {
        match event {
            FileMenuComponentEvent::OpenFileClicked => {
                self.app_scope
                    .execute(FileOpenCommand::new(self.app_scope.clone()));
            }
        }
    }
}

impl<
        TE: Debug + Send + 'static,
        EE: Debug + 'static,
        T: ClientTransport<TE>,
        E: External<Error = EE>,
    > UIComponent for FileMenuComponent<TE, EE, T, E>
{
    fn draw(&mut self, ui: &mut Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("Open").clicked() {
                self.on(FileMenuComponentEvent::OpenFileClicked);
                ui.close_menu();
            }
        });
    }
}
