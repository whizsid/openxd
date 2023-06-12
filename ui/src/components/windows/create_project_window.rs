use std::{fmt::Debug, rc::Rc};

use egui::{Align2, Pos2, TextEdit, Window};
use egui_extras::Size;
use egui_grid::GridBuilder;

use regex::Regex;

use crate::{
    client::ClientTransport,
    components::TopLevelUIComponent,
    external::External,
    scopes::{ApplicationScope, CreateProjectWindowScope}, commands::file::create_project::CreateProjectCommand,
};

pub struct CreateProjectWindow<
    TE: Debug + Send,
    EE: Debug,
    T: ClientTransport<TE>,
    E: External<Error = EE>,
> {
    scope: CreateProjectWindowScope<TE, T>,
    app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
}

impl<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>>
    CreateProjectWindow<TE, EE, T, E>
{
    pub fn new(
        scope: CreateProjectWindowScope<TE, T>,
        app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
    ) -> CreateProjectWindow<TE, EE, T, E> {
        Self { scope, app_scope }
    }
}

impl<TE: Debug + Send + 'static, EE: Debug + 'static, T: ClientTransport<TE>, E: External<Error = EE>>
    TopLevelUIComponent for CreateProjectWindow<TE, EE, T, E>
{
    fn draw(&mut self, ctx: &egui::Context) {
        let create_project_dialog_opened = self.app_scope.state().is_new_project_dialog_opened();
        let mut create_project_dialog_open = create_project_dialog_opened;
        let screen = ctx.screen_rect();

        Window::new("Create A Project")
            .movable(false)
            .pivot(Align2::CENTER_CENTER)
            .current_pos(Pos2::new(screen.right() / 2.0, screen.bottom() / 2.0))
            .open(&mut create_project_dialog_open)
            .collapsible(false)
            .show(ctx, |ui| {
                GridBuilder::new()
                    .new_row(Size::exact(26.0))
                    .cell(Size::exact(100.0))
                    .cell(Size::remainder())
                    .new_row(Size::exact(26.0))
                    .cell(Size::remainder())
                    .cell(Size::exact(50.0))
                    .show(ui, |mut grid| {
                        grid.cell(|ui| {
                            ui.label("Project Name");
                        });
                        let reg_name = Regex::new(r"^[a-zA-Z0-9\s]*$").unwrap();
                        let valid_name = reg_name.is_match(&self.scope.state().get_project_name());
                        grid.cell(|ui| {
                            let mut name_str = self.scope.state().get_project_name().clone();
                            let text_edit = ui.add(TextEdit::singleline(&mut name_str));
                            if text_edit.changed() {
                                self.scope.state_mut().change_project_name(name_str.clone());
                            }
                        });
                        grid.cell(|ui| {
                            if !valid_name {
                                ui.label("Project name should only contains numbers, characters and spaces");
                            }
                        });
                        grid.cell(|ui| {
                            ui.set_enabled(valid_name);
                            if ui.button("Create").clicked() {
                                self.app_scope.execute(CreateProjectCommand::new(self.app_scope.clone(), self.scope.state().get_project_name()));
                                self.scope.state_mut().change_project_name(String::new());
                                self.app_scope.state_mut().close_new_project_dialog();
                            }
                        })
                    });
            });

        if create_project_dialog_opened && !create_project_dialog_open {
            self.app_scope.state_mut().close_new_project_dialog();
        }
    }
}
