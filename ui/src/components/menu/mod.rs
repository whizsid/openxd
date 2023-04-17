use std::{cell::RefCell, fmt::Debug, rc::Rc, sync::Arc};

use egui::Ui;
use futures::{lock::Mutex, Sink, Stream};

use crate::{client::Client, remote_cache::RemoteCache, state::AppState};

use self::file_menu::FileMenuComponent;

mod file_menu;

pub struct MenuComponent<
    TE: Debug + 'static,
    CE: Debug,
    T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = TE> + Unpin + Send + 'static,
    C: RemoteCache<Error = CE> + Send + Sync + 'static,
> {
    file_menu: FileMenuComponent<TE, CE, T, C>,
}

impl<
        TE: Debug + 'static,
        CE: Debug,
        T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = TE> + Unpin + Send + 'static,
        C: RemoteCache<Error = CE> + Send + Sync + 'static,
    > MenuComponent<TE, CE, T, C>
{
    pub fn new(
        app_state: Rc<RefCell<AppState>>,
        client: Arc<Mutex<Client<TE, T>>>,
        remote_cache: Arc<C>,
    ) -> Self {
        MenuComponent {
            file_menu: FileMenuComponent::new(
                app_state,
                client,
                remote_cache,
            ),
        }
    }

    pub fn update(&mut self, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            self.file_menu.update(ui);
        });
    }
}
