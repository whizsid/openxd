use std::{fmt::Debug, rc::Rc};

use crate::{client::ClientTransport, external::External, scopes::ApplicationScope};

use super::UIComponent;

pub struct StatusBarComponent<
    TE: Debug + Send,
    EE: Debug,
    T: ClientTransport<TE>,
    E: External<Error = EE>,
> {
    app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
}

impl <
    TE: Debug + Send,
    EE: Debug,
    T: ClientTransport<TE>,
    E: External<Error = EE>,
> StatusBarComponent<TE, EE, T, E> {
    pub fn new(app_scope: Rc<ApplicationScope<TE, EE, T, E>>) -> StatusBarComponent<TE, EE, T, E> {
        StatusBarComponent { app_scope }
    }
}

impl <
    TE: Debug + Send,
    EE: Debug,
    T: ClientTransport<TE>,
    E: External<Error = EE>,
> UIComponent for StatusBarComponent<TE, EE, T, E> {
    fn draw(&mut self, ui: &mut egui::Ui) {
        let txt = self.app_scope.state().status_message();

        if let Some(txt) = &txt.clone() {
            ui.label(txt);
        }
    }
}
