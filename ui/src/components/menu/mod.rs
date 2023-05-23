use std::{fmt::Debug, rc::Rc};

use egui::Ui;

use crate::{client::ClientTransport, external::External, scopes::ApplicationScope};

use self::file_menu::FileMenuComponent;

use super::UIComponent;

mod file_menu;

pub struct MenuComponent<
    TE: Debug + Send + 'static,
    EE: Debug +'static,
    T: ClientTransport<TE>,
    E: External<Error = EE>,
> {
    file_menu: FileMenuComponent<TE, EE, T, E>,
    app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
}

impl<
        TE: Debug + Send + 'static,
        EE: Debug + 'static,
        T: ClientTransport<TE>,
        E: External<Error = EE>,
    > MenuComponent<TE, EE, T, E>
{
    pub fn new(app_scope: Rc<ApplicationScope<TE, EE, T, E>>) -> Self {
        MenuComponent {
            file_menu: FileMenuComponent::new(app_scope.clone()),
            app_scope,
        }
    }
}

impl<
        TE: Debug + Send + 'static,
        EE: Debug + 'static,
        T: ClientTransport<TE>,
        E: External<Error = EE>,
    > UIComponent for MenuComponent<TE, EE, T, E>
{
    fn draw(&mut self, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            self.file_menu.draw(ui);
        });
    }
}
