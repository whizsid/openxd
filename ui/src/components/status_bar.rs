use std::{fmt::Debug, rc::Rc};

use crate::{client::ClientTransport, remote_cache::RemoteCache, scopes::ApplicationScope};

use super::UIComponent;

pub struct StatusBarComponent<
    TE: Debug + Send,
    CE: Debug,
    T: ClientTransport<TE>,
    C: RemoteCache<Error = CE>,
> {
    app_scope: Rc<ApplicationScope<TE, CE, T, C>>,
}

impl <
    TE: Debug + Send,
    CE: Debug,
    T: ClientTransport<TE>,
    C: RemoteCache<Error = CE>,
> StatusBarComponent<TE, CE, T, C> {
    pub fn new(app_scope: Rc<ApplicationScope<TE, CE, T, C>>) -> StatusBarComponent<TE, CE, T, C> {
        StatusBarComponent { app_scope }
    }
}

impl <
    TE: Debug + Send,
    CE: Debug,
    T: ClientTransport<TE>,
    C: RemoteCache<Error = CE>,
> UIComponent for StatusBarComponent<TE, CE, T, C> {
    fn draw(&mut self, ui: &mut egui::Ui) {
        let txt = self.app_scope.state().status_message();

        if let Some(txt) = &txt.clone() {
            ui.label(txt);
        }
    }
}
