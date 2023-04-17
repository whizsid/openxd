use std::{cell::RefCell, fmt::Debug, rc::Rc, sync::Arc};

use egui::{CentralPanel, Context, TopBottomPanel};
use futures::lock::Mutex;
use futures::{Sink, Stream};

use crate::components::menu::MenuComponent;
use crate::remote_cache::RemoteCache;
use crate::{client::Client, state::AppState};

pub struct Ui<
    TE: Debug + 'static,
    CE: Debug,
    T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = TE> + Unpin + Send + 'static,
    C: RemoteCache<Error = CE> + Send + Sync + 'static,
> {
    // Application wide state
    state: Rc<RefCell<AppState>>,

    // Componentes
    menu_component: MenuComponent<TE, CE, T, C>,
}

impl<
        TE: Debug + 'static,
        CE: Debug,
        T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = TE> + Unpin + Send + 'static,
        C: RemoteCache<Error = CE> + Send + Sync + 'static,
    > Ui<TE, CE, T, C>
{
    pub fn new(transport: T, remote_cache: C) -> Self {
        let arc_client = Arc::new(Mutex::new(Client::new(transport)));
        let arc_remote_cache = Arc::new(remote_cache);
        let app_state = Rc::new(RefCell::new(AppState::new()));
        Self {
            state: app_state.clone(),
            menu_component: MenuComponent::new(app_state, arc_client, arc_remote_cache),
        }
    }

    // Updatng the UI in one iteration in the event loop
    pub fn update(&mut self, ctx: &Context) {
        TopBottomPanel::top("menu-bar").show(ctx, |ui| {
            ui.add_enabled_ui(!self.state.borrow().is_main_ui_disabled(), |ui| {
                self.menu_component.update(ui);
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!self.state.borrow().is_main_ui_disabled(), |ui| {
                ui.heading("Hello World!");
            })
        });
    }
}
