use std::{fmt::Debug, rc::Rc};

use egui::Ui;

use crate::{client::ClientTransport, cache::Cache, scopes::ApplicationScope};

use self::file_menu::FileMenuComponent;

use super::UIComponent;

mod file_menu;

pub struct MenuComponent<
    TE: Debug + Send + 'static,
    CE: Debug +'static,
    T: ClientTransport<TE>,
    C: Cache<Error = CE>,
> {
    file_menu: FileMenuComponent<TE, CE, T, C>,
    app_scope: Rc<ApplicationScope<TE, CE, T, C>>,
}

impl<
        TE: Debug + Send + 'static,
        CE: Debug + 'static,
        T: ClientTransport<TE>,
        C: Cache<Error = CE>,
    > MenuComponent<TE, CE, T, C>
{
    pub fn new(app_scope: Rc<ApplicationScope<TE, CE, T, C>>) -> Self {
        MenuComponent {
            file_menu: FileMenuComponent::new(app_scope.clone()),
            app_scope,
        }
    }
}

impl<
        TE: Debug + Send + 'static,
        CE: Debug + 'static,
        T: ClientTransport<TE>,
        C: Cache<Error = CE>,
    > UIComponent for MenuComponent<TE, CE, T, C>
{
    fn draw(&mut self, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            self.file_menu.draw(ui);
        });
    }
}
