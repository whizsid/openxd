use std::{fmt::Debug, rc::Rc};

use egui::Ui;

use crate::{
    client::ClientTransport, commands::file::open_file::FileOpenCommand, components::UIComponent,
    remote_cache::RemoteCache, scopes::ApplicationScope,
};

pub enum FileMenuComponentEvent {
    OpenFileClicked,
}

pub struct FileMenuComponent<
    TE: Debug + Send + 'static,
    CE: Debug + 'static,
    T: ClientTransport<TE>,
    C: RemoteCache<Error = CE>,
> {
    app_scope: Rc<ApplicationScope<TE, CE, T, C>>,
}

impl<
        TE: Debug + Send + 'static,
        CE: Debug + 'static,
        T: ClientTransport<TE>,
        C: RemoteCache<Error = CE>,
    > FileMenuComponent<TE, CE, T, C>
{
    pub fn new(app_scope: Rc<ApplicationScope<TE, CE, T, C>>) -> Self {
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
        CE: Debug + 'static,
        T: ClientTransport<TE>,
        C: RemoteCache<Error = CE>,
    > UIComponent for FileMenuComponent<TE, CE, T, C>
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
