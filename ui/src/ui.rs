use std::{fmt::Debug, rc::Rc};

use egui::{CentralPanel, Context, TopBottomPanel};

use crate::client::ClientTransport;
use crate::components::UIComponent;
use crate::components::menu::MenuComponent;
use crate::remote_cache::RemoteCache;
use crate::scopes::ApplicationScope;

pub struct Ui<
    TE: Debug + 'static,
    CE: Debug + 'static,
    T: ClientTransport<TE> + Send + 'static,
    C: RemoteCache<Error = CE> + Send + Sync + 'static,
> {
    scope: Rc<ApplicationScope<TE, CE, T, C>>,
    // Componentes
    menu_component: MenuComponent<TE, CE, T, C>,
}

impl<
        TE: Debug + 'static,
        CE: Debug + 'static,
        T: ClientTransport<TE> + Send + 'static,
        C: RemoteCache<Error = CE> + Send + Sync + 'static,
    > Ui<TE, CE, T, C>
{
    pub fn new(transport: T, remote_cache: C) -> Self {
        let app_scope = Rc::new(ApplicationScope::new(transport, remote_cache));
        
        Self {
            scope: app_scope.clone(),
            menu_component: MenuComponent::new(app_scope),
        }
    }

    // Updatng the UI in one iteration in the event loop
    pub fn update(&mut self, ctx: &Context) {
        let main_ui_disabled = self.scope.state().is_main_ui_disabled();

        self.scope.update_cmd_executor();

        TopBottomPanel::top("menu-bar").show(ctx, |ui| {
            ui.add_enabled_ui(!main_ui_disabled, |ui| {
                self.menu_component.draw(ui);
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!main_ui_disabled, |ui| {
                ui.heading("Hello World!");
            })
        });
    }
}
