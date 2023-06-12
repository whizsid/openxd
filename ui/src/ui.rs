//! This module will control all the other UI components
//!
//! All the components defined in `components` module should be linked
//! here.

use std::{fmt::Debug, rc::Rc};

use egui::{CentralPanel, Context, TopBottomPanel, Window};

use crate::client::ClientTransport;
use crate::components::dialog_container::DialogContainerComponent;
use crate::components::menu::MenuComponent;
use crate::components::status_bar::StatusBarComponent;
use crate::components::windows::create_project_window::CreateProjectWindow;
use crate::components::{TopLevelUIComponent, UIComponent};
use crate::external::External;
use crate::scopes::{ApplicationScope, CreateProjectWindowScope};

pub struct Ui<
    TE: Debug + Send + 'static,
    EE: Debug + 'static,
    T: ClientTransport<TE>,
    E: External<Error = EE>,
> {
    scope: Rc<ApplicationScope<TE, EE, T, E>>,
    // Componentes
    menu_component: MenuComponent<TE, EE, T, E>,
    status_bar_component: StatusBarComponent<TE, EE, T, E>,
    dialog_container_component: DialogContainerComponent<TE, EE, T, E>,
    create_project_window: CreateProjectWindow<TE, EE, T, E>,
}

impl<
        TE: Debug + Send + 'static,
        EE: Debug + 'static,
        T: ClientTransport<TE>,
        E: External<Error = EE>,
    > Ui<TE, EE, T, E>
{
    /// Creating the main UI by passing external interfaces
    pub fn new(transport: T, external_client: E) -> Self {
        let app_scope = Rc::new(ApplicationScope::new(transport, external_client));

        Self {
            scope: app_scope.clone(),
            menu_component: MenuComponent::new(app_scope.clone()),
            status_bar_component: StatusBarComponent::new(app_scope.clone()),
            dialog_container_component: DialogContainerComponent::new(app_scope.clone()),
            create_project_window: CreateProjectWindow::new(CreateProjectWindowScope::new(
                app_scope.client(),
            ), app_scope.clone()),
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

        TopBottomPanel::bottom("status-bar")
            .exact_height(22.00)
            .show(ctx, |ui| {
                self.status_bar_component.draw(ui);
            });

        // Dialogs
        self.dialog_container_component.draw(ctx);

        // Windows
        self.create_project_window.draw(ctx);

        CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!main_ui_disabled, |ui| {
                ui.heading("Hello World!");
            })
        });
    }
}
