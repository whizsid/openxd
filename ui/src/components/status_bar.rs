use crate::scopes::ApplicationScope;

use super::UIComponent;

pub struct StatusBarComponent {
    app_scope: ApplicationScope,
}

impl StatusBarComponent {
    pub fn new(app_scope: ApplicationScope) -> StatusBarComponent {
        StatusBarComponent { app_scope }
    }
}

impl UIComponent for StatusBarComponent {
    fn draw(&mut self, ui: &mut egui::Ui) {
        let txt = self.app_scope.state().status_message();

        if let Some(txt) = &txt.clone() {
            ui.label(txt);
        }
    }
}
