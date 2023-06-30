use egui::Ui;

use crate::scopes::ApplicationScope;

use self::file_menu::FileMenuComponent;

use super::UIComponent;

mod file_menu;

pub struct MenuComponent {
    file_menu: FileMenuComponent,
    _app_scope: ApplicationScope,
}

impl MenuComponent {
    pub fn new(app_scope: ApplicationScope) -> Self {
        MenuComponent {
            file_menu: FileMenuComponent::new(app_scope.clone()),
            _app_scope: app_scope,
        }
    }
}

impl UIComponent for MenuComponent {
    fn draw(&mut self, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            self.file_menu.draw(ui);
        });
    }
}
