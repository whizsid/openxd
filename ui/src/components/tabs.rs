use std::{fmt::Debug, rc::Rc};

use crate::{client::ClientTransport, external::External, scopes::ApplicationScope};

pub struct ProjectsTabViewer<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>> {
    app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
}

impl<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>>
    ProjectsTabViewer<TE, EE, T, E>
{
    pub fn new(app_scope: Rc<ApplicationScope<TE, EE, T, E>>) -> ProjectsTabViewer<TE, EE, T, E> {
        ProjectsTabViewer { app_scope }
    }
}

impl<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>>
    egui_dock::TabViewer for ProjectsTabViewer<TE, EE, T, E>
{
    type Tab = usize;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {}

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

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        true
    }
}


pub enum LeftPanelTabKind {
    Layers,
    Components
}

pub struct LeftPanelTabViewer <TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>> {
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
            LeftPanelTabKind::Components => "Components".into()
        }
    }
}

pub enum RightPanelTabKind {
    Tool,
    Transform,
    Appearance,
    Properties
}

pub struct RightPanelTabViewer <TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>> {
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

