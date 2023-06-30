//! This module will control all the other UI components
//!
//! All the components defined in `components` module should be linked
//! here.

use std::sync::Arc;

use egui::{
    CentralPanel, Context, FontData, FontDefinitions, FontFamily, Id, SidePanel, TopBottomPanel,
};
use egui_dock::DockArea;
use futures::lock::Mutex;

use crate::client::Client;
use crate::components::dialog_container::DialogContainerComponent;
use crate::components::menu::MenuComponent;
use crate::components::quick_icons::QuickIconsComponent;
use crate::components::status_bar::StatusBarComponent;
use crate::components::tabs::{LeftPanelTabViewer, ProjectsTabViewer, RightPanelTabViewer};
use crate::components::windows::create_project_window::CreateProjectWindow;
use crate::components::{TopLevelUIComponent, UIComponent};
use crate::external::External;
use crate::scopes::{ApplicationScope, CreateProjectWindowScope};

pub struct Ui {
    scope: ApplicationScope,
    // Componentes
    menu_component: MenuComponent,
    status_bar_component: StatusBarComponent,
    dialog_container_component: DialogContainerComponent,
    create_project_window: CreateProjectWindow,
    tab_viewer: ProjectsTabViewer,
    left_panel_tab_viewer: LeftPanelTabViewer,
    right_panel_tab_viewer: RightPanelTabViewer,
    quick_icons_component: QuickIconsComponent,
}

impl Ui {
    /// Creating the main UI by passing external interfaces
    pub fn new(
        ctx: &Context,
        gl: Arc<glow::Context>,
        client: Box<dyn Client + 'static>,
        external_client: Box<dyn External + 'static>,
    ) -> Self {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "icon-font".to_owned(),
            FontData::from_static(include_bytes!("../fonts/icons.ttf")),
        );
        fonts.families.insert(
            FontFamily::Name("system-ui".into()),
            vec!["icon-font".into()],
        );
        ctx.set_fonts(fonts);

        let app_scope = ApplicationScope::new(
                    Arc::new(Mutex::new(client)),
                    Arc::new(external_client)
                );

        let quick_icons_component = QuickIconsComponent::new(app_scope.clone(), &ctx.style());

        Self {
            menu_component: MenuComponent::new(app_scope.clone()),
            status_bar_component: StatusBarComponent::new(app_scope.clone()),
            dialog_container_component: DialogContainerComponent::new(app_scope.clone()),
            create_project_window: CreateProjectWindow::new(
                CreateProjectWindowScope::new(),
                app_scope.clone(),
            ),
            tab_viewer: ProjectsTabViewer::new(app_scope.clone(), gl),
            left_panel_tab_viewer: LeftPanelTabViewer::new(app_scope.clone()),
            right_panel_tab_viewer: RightPanelTabViewer::new(app_scope.clone()),
            quick_icons_component,
            scope: app_scope,
        }
    }

    pub fn exit(&mut self, gl: Option<&glow::Context>) {
        self.tab_viewer.exit(gl);
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

        SidePanel::left("left-panel").show(ctx, |ui| {
            let height = ui.available_size().y;
            ui.horizontal(|ui| {
                ui.set_height(height);
                ui.vertical(|ui| {
                    self.quick_icons_component.draw(ui);
                });
                ui.vertical(|ui| {
                    let mut tree = self.scope.left_panel_tree();
                    DockArea::new(&mut *tree)
                        .id(Id::new("left-panel-dock"))
                        .style(egui_dock::Style::from_egui(ui.style().as_ref()))
                        .show_inside(ui, &mut self.left_panel_tab_viewer);
                });
            });
        });

        SidePanel::right("right-panel").show(ctx, |ui| {
            let mut tree = self.scope.right_panel_tree();
            egui_dock::DockArea::new(&mut *tree)
                .id(Id::new("right-panel-dock"))
                .style(egui_dock::Style::from_egui(ui.style().as_ref()))
                .show_inside(ui, &mut self.right_panel_tab_viewer);
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!main_ui_disabled, |ui| {
                let mut projects_tree = self.scope.projects_tree();
                egui_dock::DockArea::new(&mut *projects_tree)
                    .id(Id::new("central-panel-dock"))
                    .draggable_tabs(false)
                    .style(egui_dock::Style::from_egui(ui.style().as_ref()))
                    .show_inside(ui, &mut self.tab_viewer);
            })
        });
    }
}
