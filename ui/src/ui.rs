//! This module will control all the other UI components
//!
//! All the components defined in `components` module should be linked
//! here.

use std::{fmt::Debug, rc::Rc};

use egui::{CentralPanel, Context, TopBottomPanel, Area, pos2};

use crate::client::ClientTransport;
use crate::components::UIComponent;
use crate::components::menu::MenuComponent;
use crate::components::status_bar::StatusBarComponent;
use crate::remote_cache::RemoteCache;
use crate::scopes::ApplicationScope;

pub struct Ui<
    TE: Debug + Send + 'static,
    CE: Debug + 'static,
    T: ClientTransport<TE>,
    C: RemoteCache<Error = CE>,
> {
    scope: Rc<ApplicationScope<TE, CE, T, C>>,
    // Componentes
    menu_component: MenuComponent<TE, CE, T, C>,
    status_bar_component: StatusBarComponent<TE, CE, T, C>,
}

impl<
        TE: Debug + Send + 'static,
        CE: Debug + 'static,
        T: ClientTransport<TE>,
        C: RemoteCache<Error = CE>,
    > Ui<TE, CE, T, C>
{
    /// Creating the main UI by passing external interfaces
    pub fn new(transport: T, remote_cache: C) -> Self {
        let app_scope = Rc::new(ApplicationScope::new(transport, remote_cache));
        
        Self {
            scope: app_scope.clone(),
            menu_component: MenuComponent::new(app_scope.clone()),
            status_bar_component: StatusBarComponent::new(app_scope)
        }
    }

    /// Updating the components and command statuses in a one iteration in event loop.
    ///
    /// Please refer the [eframe::App::update](https://docs.rs/eframe/latest/eframe/trait.App.html#tymethod.update)
    /// method.
    pub fn update(&mut self, ctx: &Context) {
        let main_ui_disabled = self.scope.state().is_main_ui_disabled();

        self.scope.update_cmd_executor();

        TopBottomPanel::top("menu-bar").show(ctx, |ui| {
            ui.add_enabled_ui(!main_ui_disabled, |ui| {
                self.menu_component.draw(ui);
            });
        });

        TopBottomPanel::bottom("status-bar").exact_height(22.00).show(ctx, |ui|{
            self.status_bar_component.draw(ui);
        });

        let state = self.scope.state();
        let dialog_len = state.len_dialogs();
        for i in 0..dialog_len {
            let dialog = state.get_dialog(i).unwrap();
            Area::new(format!("dialog-{}", i)).fixed_pos(pos2(-0.01,-0.01)).show(ctx, |ui| {
                ui.label(dialog.message());
            });
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!main_ui_disabled, |ui| {
                ui.heading("Hello World!");
            })
        });


    }
}
