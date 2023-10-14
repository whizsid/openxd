//! Application Scopes
//!
//! Some shared values/interfaces only required for a specific scope (for canvas, for menu bar).
//! But some values/interfaces are application wide. So we have to redefine what are the required
//! parameters for each component. Those scopes will avoid those redefinitions.
use std::{sync::Arc, cell::{RefCell, Ref, RefMut}, rc::Rc};

use egui_dock::DockState;
use futures::lock::Mutex;
use transport::vo::Screen;

use crate::{
    client::Client,
    commands::{Command, Executor},
    components::tabs::{LeftPanelTabKind, RightPanelTabKind},
    external::External,
    state::{AppState, CreateProjectWindowState},
};

/// Application wide scope
#[derive(Clone)]
pub struct ApplicationScope {
    client: Arc<Mutex<Box<dyn Client>>>,
    external_client: Arc<Box<dyn External>>,
    command_executor: Rc<RefCell<Executor>>,
    state: Rc<RefCell<AppState>>,
    projects_tree: Rc<RefCell<DockState<usize>>>,
    left_panel_tree: Rc<RefCell<DockState<LeftPanelTabKind>>>,
    right_panel_tree: Rc<RefCell<DockState<RightPanelTabKind>>>,
}

impl ApplicationScope {
    pub fn new(
        client: Arc<Mutex<Box<dyn Client>>>,
        external_client: Arc<Box<dyn External>>,
    ) -> ApplicationScope {
        let tree = DockState::new(vec![]);
        let left_panel_tree =
            DockState::new(vec![LeftPanelTabKind::Layers, LeftPanelTabKind::Components]);
        let right_panel_tree = DockState::new(vec![
            RightPanelTabKind::Tool,
            RightPanelTabKind::Appearance,
            RightPanelTabKind::Properties,
        ]);

        ApplicationScope {
            client,
            external_client,
            command_executor: Rc::new(RefCell::new(Executor::new())),
            state: Rc::new(RefCell::new(AppState::new())),
            projects_tree: Rc::new(RefCell::new(tree)),
            left_panel_tree: Rc::new(RefCell::new(left_panel_tree)),
            right_panel_tree: Rc::new(RefCell::new(right_panel_tree)),
        }
    }

    /// Getter for a non mutable reference to application wide state
    pub fn state(&self) ->  Ref<AppState> {
        self.state.borrow()
    }

    /// Getter for a mutable reference to application wide state
    pub fn state_mut(&self) -> RefMut<AppState> {
        self.state.borrow_mut()
    }

    /// Executing a command using command executor
    pub fn execute<CMD: Command + 'static>(&self, cmd: CMD) {
        self.command_executor.borrow_mut().execute(cmd);
    }

    /// Executing a boxed command using command executor
    pub fn execute_boxed(&self, cmd: Box<dyn Command + 'static>) {
        self.command_executor.borrow_mut().execute_boxed(cmd);
    }

    /// Getting a reference for the remote cache
    pub fn external_client(&self) -> Arc<Box<dyn External>> {
        self.external_client.clone()
    }

    /// Getting a reference for the client
    pub fn client(&self) -> Arc<Mutex<Box<dyn Client>>> {
        self.client.clone()
    }

    /// Updating the command executor
    ///
    /// This method will run in the egui event loop. This method will update
    /// the command status using the `poll_promise` promises.
    pub fn update_cmd_executor(&self) {
        self.command_executor.borrow_mut().update();
    }

    /// Adding a project as a tab
    pub fn add_project(&self, id: String, title: String, zoom: f64, screens: Vec<Screen>) {
        self.state.borrow_mut().add_project(id, title, zoom, screens);
        let count = self.state.borrow().tab_count();
        self.projects_tree.borrow_mut().push_to_first_leaf(count - 1);
    }

    pub fn projects_tree(&self) -> RefMut<DockState<usize>> {
        self.projects_tree.borrow_mut()
    }

    pub fn left_panel_tree(&self) -> RefMut<DockState<LeftPanelTabKind>> {
        self.left_panel_tree.borrow_mut()
    }

    pub fn right_panel_tree(&self) -> RefMut<DockState<RightPanelTabKind>> {
        self.right_panel_tree.borrow_mut()
    }

    pub fn remove_tab(&self, tab_idx: usize) {
        let tab_loc = self.projects_tree.borrow().find_tab(&tab_idx).unwrap();
        self.projects_tree.borrow_mut().remove_tab(tab_loc);

        self.state.borrow_mut().remove_tab(tab_idx);
    }
}

pub struct CreateProjectWindowScope {
    state: CreateProjectWindowState,
}

impl CreateProjectWindowScope {
    pub fn new() -> CreateProjectWindowScope {
        CreateProjectWindowScope {
            state: CreateProjectWindowState::new(),
        }
    }

    /// Getter for a non mutable reference to application wide state
    pub fn state(&self) -> &CreateProjectWindowState {
        &self.state
    }

    /// Getter for a mutable reference to application wide state
    pub fn state_mut(&mut self) -> &mut CreateProjectWindowState {
        &mut self.state
    }
}
