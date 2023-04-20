use std::{fmt::Debug, rc::Rc};

use egui::Ui;
use futures::{Sink, Stream};

use crate::{client::ClientTransport, remote_cache::RemoteCache, scopes::ApplicationScope};

use self::file_menu::FileMenuComponent;

use super::UIComponent;

mod file_menu;

pub struct MenuComponent<
    TE: Debug + 'static,
    CE: Debug +'static,
    T: ClientTransport<TE> + Send + 'static,
    C: RemoteCache<Error = CE> + Send + Sync + 'static,
> {
    file_menu: FileMenuComponent<TE, CE, T, C>,
    app_scope: Rc<ApplicationScope<TE, CE, T, C>>,
}

impl<
        TE: Debug + 'static,
        CE: Debug + 'static,
        T: ClientTransport<TE> + Send + 'static,
        C: RemoteCache<Error = CE> + Send + Sync + 'static,
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
        TE: Debug + 'static,
        CE: Debug + 'static,
        T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = TE> + Unpin + Send + 'static,
        C: RemoteCache<Error = CE> + Send + Sync + 'static,
    > UIComponent for MenuComponent<TE, CE, T, C>
{
    fn draw(&mut self, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            self.file_menu.draw(ui);
        });
    }
}
