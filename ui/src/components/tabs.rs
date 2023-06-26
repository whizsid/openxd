use std::{fmt::Debug, rc::Rc, sync::Arc};

use egui::Frame;

use crate::{
    client::ClientTransport, commands::{tab::close_tab::TabCloseCommand, nope::NopeCommand}, external::External,
    scopes::ApplicationScope, state::Severity,
};

use super::{canvas::CanvasComponent, UIComponent};

pub struct ProjectsTabViewer<
    TE: Debug + Send + 'static,
    EE: Debug + 'static,
    T: ClientTransport<TE>,
    E: External<Error = EE>,
> {
    app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
    canvas_component: CanvasComponent,
    last_tab: usize,
}

impl<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>>
    ProjectsTabViewer<TE, EE, T, E>
{
    pub fn new(app_scope: Rc<ApplicationScope<TE, EE, T, E>>, gl: Arc<glow::Context>) -> ProjectsTabViewer<TE, EE, T, E> {
        ProjectsTabViewer { app_scope, canvas_component: CanvasComponent::new(gl), last_tab: 0 }
    }

    pub fn exit(&mut self, gl: Option<&glow::Context>) {
        self.canvas_component.exit(gl);
    }
}

impl<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>>
    egui_dock::TabViewer for ProjectsTabViewer<TE, EE, T, E>
{
    type Tab = usize;

    fn ui(&mut self, ui: &mut egui::Ui, tab_idx: &mut Self::Tab) {
        let tab = self.app_scope.state().tab(*tab_idx);
        if self.last_tab != *tab_idx {
            if let Some(_tab) = tab {
                self.canvas_component.change_tab(*tab_idx);
            }
        }
        Frame::canvas(ui.style()).show(ui, |ui| {
            self.canvas_component.draw(ui);
        });
    }

    fn title(&mut self, i: &mut Self::Tab) -> egui::WidgetText {
        let tab_found = self.app_scope.state().tab(i.clone());
        match tab_found {
            Some(tab_found) => {
                let tab_borrowed = tab_found.borrow();
                let tab_title = tab_borrowed.title().clone();
                let saved = tab_borrowed.saved();
                if saved {
                    tab_title.into()
                } else {
                    format!("\u{23FA}  {}", tab_title).into()
                }
            }
            None => "Opening...".into(),
        }
    }

    fn on_close(&mut self, tab_idx: &mut Self::Tab) -> bool {
        let tab = self.app_scope.state().tab(tab_idx.clone()).unwrap();
        let mut borrowed_tab = tab.borrow_mut();
        if borrowed_tab.closing() {
            return false;
        }
        borrowed_tab.set_closing(true);
        if !borrowed_tab.saved() {
            let mut state_mut = self.app_scope.state_mut();
            let dialog = state_mut.add_dialog(
                Severity::Info,
                "There are some unsaved changes. Are you sure you want to close this tab?",
            );

            let cloned_app_scope = self.app_scope.clone();
            let cloned_tab_idx = tab_idx.clone();
            drop(borrowed_tab);
            dialog.add_button(Severity::Error, "Yes").on_click(move || {
                Box::new(TabCloseCommand::new(
                    cloned_app_scope,
                    cloned_tab_idx,
                ))
            });
            let app_scope = self.app_scope.clone();
            let cloned_tab_idx = tab_idx.clone();
            dialog.on_close(move || {
                let state = app_scope.state();
                let tab = state.tab(cloned_tab_idx).unwrap();
                let mut borrowed_tab = tab.borrow_mut();
                borrowed_tab.set_closing(false);
                Box::new(NopeCommand::new())
            });
        } else {
            drop(borrowed_tab);
            self.app_scope.execute(TabCloseCommand::new(
                self.app_scope.clone(),
                tab_idx.clone(),
            ));
        }

        false
    }
}

pub enum LeftPanelTabKind {
    Layers,
    Components,
}

pub struct LeftPanelTabViewer<
    TE: Debug + Send,
    EE: Debug,
    T: ClientTransport<TE>,
    E: External<Error = EE>,
> {
    app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
}

impl<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>>
    LeftPanelTabViewer<TE, EE, T, E>
{
    pub fn new(app_scope: Rc<ApplicationScope<TE, EE, T, E>>) -> LeftPanelTabViewer<TE, EE, T, E> {
        LeftPanelTabViewer { app_scope }
    }
}

impl<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>>
    egui_dock::TabViewer for LeftPanelTabViewer<TE, EE, T, E>
{
    type Tab = LeftPanelTabKind;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {}

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            LeftPanelTabKind::Layers => "Layers".into(),
            LeftPanelTabKind::Components => "Components".into(),
        }
    }
}

pub enum RightPanelTabKind {
    Tool,
    Transform,
    Appearance,
    Properties,
}

pub struct RightPanelTabViewer<
    TE: Debug + Send,
    EE: Debug,
    T: ClientTransport<TE>,
    E: External<Error = EE>,
> {
    app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
}

impl<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>>
    RightPanelTabViewer<TE, EE, T, E>
{
    pub fn new(app_scope: Rc<ApplicationScope<TE, EE, T, E>>) -> RightPanelTabViewer<TE, EE, T, E> {
        RightPanelTabViewer { app_scope }
    }
}

impl<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>>
    egui_dock::TabViewer for RightPanelTabViewer<TE, EE, T, E>
{
    type Tab = RightPanelTabKind;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {}

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            RightPanelTabKind::Tool => "Tool".into(),
            RightPanelTabKind::Transform => "Transform".into(),
            RightPanelTabKind::Appearance => "Appearance".into(),
            RightPanelTabKind::Properties => "Properties".into(),
        }
    }
}
