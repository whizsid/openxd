use egui::{Align2, Pos2, TextEdit, Window};
use egui_extras::Size;
use egui_grid::GridBuilder;

use regex::Regex;

use crate::{
    commands::file::create_project::CreateProjectCommand,
    components::TopLevelUIComponent,
    scopes::{ApplicationScope, CreateProjectWindowScope},
};

pub struct CreateProjectWindow {
    scope: CreateProjectWindowScope,
    app_scope: ApplicationScope,
}

impl CreateProjectWindow {
    pub fn new(
        scope: CreateProjectWindowScope,
        app_scope: ApplicationScope,
    ) -> CreateProjectWindow {
        Self { scope, app_scope }
    }
}

impl TopLevelUIComponent for CreateProjectWindow {
    fn draw(&mut self, ctx: &egui::Context) {
        let create_project_dialog_opened = self.app_scope.state().create_project_window().is_open();
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
                                let project_name = self.scope.state().get_project_name();
                                let create_project_command = CreateProjectCommand::new(self.app_scope.clone(), project_name.clone() );
                                self.app_scope.execute(create_project_command);
                                let mut state_mut = self.app_scope.state_mut();
                                let create_project_state = state_mut.create_project_window_mut();
                                create_project_state.change_project_name(String::new());
                                create_project_state.close();
                            }
                        })
                    });
            });

        if create_project_dialog_opened && !create_project_dialog_open {
            self.app_scope
                .state_mut()
                .create_project_window_mut()
                .close();
        }
    }
}
