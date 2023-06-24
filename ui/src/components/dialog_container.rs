use std::{fmt::Debug, rc::Rc};

use egui::{pos2, Align, Area, Frame, Layout, Margin};

use crate::{client::ClientTransport, external::External, scopes::ApplicationScope};

use super::TopLevelUIComponent;

const DIALOG_WIDTH: f32 = 160.00;
const DIALOG_MARGIN_X: f32 = 8.0;
const DIALOG_MARGIN_Y: f32 = 8.0;

pub struct DialogContainerComponent<
    TE: Debug + Send,
    EE: Debug,
    T: ClientTransport<TE>,
    E: External<Error = EE>,
> {
    app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
}

impl<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>>
    DialogContainerComponent<TE, EE, T, E>
{
    pub fn new(
        app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
    ) -> DialogContainerComponent<TE, EE, T, E> {
        DialogContainerComponent { app_scope }
    }

    pub fn close_button_clicked(&self, dialog_id: usize) {
        let dialog_opt = self.app_scope.state_mut().remove_dialog(dialog_id);
        if let Some(dialog) = dialog_opt {
            let close_cmd_opt = dialog.create_close_command();

            if let Some(close_cmd) = close_cmd_opt {
                self.app_scope.execute_boxed(close_cmd);
            }
        }
    }

    pub fn custom_button_clicked(&self, dialog_id: usize, btn_id: usize) {
        let state = self.app_scope.state();
        let exist_btn = state.get_dialog(dialog_id);
        match exist_btn {
            Some(btn_exist) => match btn_exist.button(btn_id) {
                Some(_) => {}
                None => {
                    return;
                }
            },
            None => {
                return;
            }
        };
        drop(state);

        let mut dialog = self.app_scope.state_mut().remove_dialog(dialog_id).unwrap();
        let btn = dialog.pop_button(btn_id).unwrap();
        let cmd_opt = btn.create_command();

        if let Some(cmd) = cmd_opt {
            self.app_scope.execute_boxed(cmd);
        }
    }
}

impl<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>>
    TopLevelUIComponent for DialogContainerComponent<TE, EE, T, E>
{
    fn draw(&mut self, ctx: &egui::Context) {
        let win_max = ctx.screen_rect().max;
        Area::new("dialogs")
            .movable(false)
            .interactable(true)
            .fixed_pos(pos2(win_max.x - DIALOG_WIDTH - DIALOG_MARGIN_X * 2.0, 0.0))
            .show(ctx, |ui| {
                ui.set_height(win_max.y);
                ui.set_width(DIALOG_WIDTH + DIALOG_MARGIN_X * 2.0);
                let state = self.app_scope.state();
                let dialogs = state.dialogs();
                drop(state);
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    for dialog in dialogs {
                        ui.horizontal_top(|ui| {
                            Frame::popup(ui.style())
                                .outer_margin(Margin::symmetric(0.0, DIALOG_MARGIN_Y))
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        ui.set_width(DIALOG_WIDTH);
                                        ui.label(dialog.message());

                                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                            if ui.button(format!("Close")).clicked() {
                                                self.close_button_clicked(dialog.id());
                                            }
                                            for dialog_btn in dialog.iter_buttons() {
                                                if ui.button(dialog_btn.text()).clicked() {
                                                    self.custom_button_clicked(
                                                        dialog.id(),
                                                        dialog_btn.id(),
                                                    );
                                                }
                                            }
                                        });
                                    });
                                });
                        });
                    }
                });
            });
    }
}
