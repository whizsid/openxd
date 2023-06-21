//! This module will control all the other UI components
//!
//! All the components defined in `components` module should be linked
//! here.

use std::{fmt::Debug, rc::Rc};

use egui::{CentralPanel, Context, Id, SidePanel, TopBottomPanel, FontDefinitions, FontData, FontId, FontFamily, TextStyle};
use egui_extras::RetainedImage;

use crate::client::ClientTransport;
use crate::components::dialog_container::DialogContainerComponent;
use crate::components::menu::MenuComponent;
use crate::components::status_bar::StatusBarComponent;
use crate::components::tabs::{LeftPanelTabViewer, ProjectsTabViewer, RightPanelTabViewer};
use crate::components::windows::create_project_window::CreateProjectWindow;
use crate::components::{TopLevelUIComponent, UIComponent};
use crate::external::External;
use crate::scopes::{ApplicationScope, CreateProjectWindowScope};

pub struct UIIcons {
    pub draw_rectangle: RetainedImage,
}

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
    tab_viewer: ProjectsTabViewer<TE, EE, T, E>,
    left_panel_tab_viewer: LeftPanelTabViewer<TE, EE, T, E>,
    right_panel_tab_viewer: RightPanelTabViewer<TE, EE, T, E>,
}

impl<
        TE: Debug + Send + 'static,
        EE: Debug + 'static,
        T: ClientTransport<TE>,
        E: External<Error = EE>,
    > Ui<TE, EE, T, E>
{
    /// Creating the main UI by passing external interfaces
    pub fn new(ctx: &Context, transport: T, external_client: E) -> Self {

        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert("icon-font".to_owned(), FontData::from_static(include_bytes!("../fonts/icons.ttf")));
        fonts.families.insert(FontFamily::Name("system-ui".into()), vec!["icon-font".into()]);
        ctx.set_fonts(fonts);

        let app_scope = Rc::new(ApplicationScope::new(transport, external_client));

        Self {
            scope: app_scope.clone(),
            menu_component: MenuComponent::new(app_scope.clone()),
            status_bar_component: StatusBarComponent::new(app_scope.clone()),
            dialog_container_component: DialogContainerComponent::new(app_scope.clone()),
            create_project_window: CreateProjectWindow::new(
                CreateProjectWindowScope::new(app_scope.client()),
                app_scope.clone(),
            ),
            tab_viewer: ProjectsTabViewer::new(app_scope.clone()),
            left_panel_tab_viewer: LeftPanelTabViewer::new(app_scope.clone()),
            right_panel_tab_viewer: RightPanelTabViewer::new(app_scope.clone()),
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

        SidePanel::left("left-panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.style_mut().text_styles= [(TextStyle::Button, FontId::new(40.0, FontFamily::Name("system-ui".into())))].into();
                    if ui.button("!").clicked() {
                        println!("Clicked");
                    }
                });
                let mut tree = self.scope.left_panel_tree();
                egui_dock::DockArea::new(&mut *tree)
                    .id(Id::new("left-panel-dock"))
                    .style(egui_dock::Style::from_egui(ui.style().as_ref()))
                    .show_inside(ui, &mut self.left_panel_tab_viewer);
                drop(tree);
            });
        });

        SidePanel::right("right-panel").show(ctx, |ui| {
            let mut tree = self.scope.right_panel_tree();
            egui_dock::DockArea::new(&mut *tree)
                .id(Id::new("right-panel-dock"))
                .style(egui_dock::Style::from_egui(ui.style().as_ref()))
                .show_inside(ui, &mut self.right_panel_tab_viewer);
            drop(tree);
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!main_ui_disabled, |ui| {
                let mut projects_tree = self.scope.projects_tree();
                egui_dock::DockArea::new(&mut *projects_tree)
                    .id(Id::new("central-panel-dock"))
                    .draggable_tabs(false)
                    .style(egui_dock::Style::from_egui(ui.style().as_ref()))
                    .show_inside(ui, &mut self.tab_viewer);
                drop(projects_tree);
            })
        });
    }
}
