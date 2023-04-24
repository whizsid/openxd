use std::{fmt::Debug, rc::Rc};

use crate::{client::ClientTransport, remote_cache::RemoteCache, scopes::ApplicationScope};

use super::UIComponent;

pub struct DialogComponent<
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
> DialogComponent <TE, CE, T, C> {
    pub fn new(app_scope: Rc<ApplicationScope<TE, CE, T, C>>) -> DialogComponent<TE, CE, T, C> {
        DialogComponent { app_scope }
    }
}

impl <
    TE: Debug + Send,
    CE: Debug,
    T: ClientTransport<TE>,
    C: RemoteCache<Error = CE>,
> UIComponent for DialogComponent<TE,CE,T,C> {
    fn draw(&mut self, ui: &mut egui::Ui) {
        
    }
}
