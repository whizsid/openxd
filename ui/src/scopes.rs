//! Application Scopes
//!
//! Some shared values/interfaces only required for a specific scope (for canvas, for menu bar).
//! But some values/interfaces are application wide. So we have to redefine what are the required
//! parameters for each component. Those scopes will avoid those redefinitions.
use std::{fmt::Debug, marker::PhantomData, sync::Arc, rc::Rc, cell::{RefCell, Ref, RefMut}};

use egui_dock::Tree;
use futures::lock::Mutex;
use transport::vo::Screen;

use crate::{client::{ClientTransport, Client}, external::External, commands::{Executor, Command}, state::{AppState, CreateProjectWindowState}, tab::TabInfo, components::tabs::{LeftPanelTabKind, RightPanelTabKind}};

/// Application wide scope
pub struct ApplicationScope<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>> {
    client: Arc<Mutex<Client<TE, T>>>,
    external_client: Arc<E>,
    command_executor: Rc<RefCell<Executor>>,
    state: Rc<RefCell<AppState>>,
    _phantom: PhantomData<TE>,
    projects_tree: Rc<RefCell<Tree<usize>>>,
    left_panel_tree: Rc<RefCell<Tree<LeftPanelTabKind>>>,
    right_panel_tree: Rc<RefCell<Tree<RightPanelTabKind>>>
}

impl <TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>> ApplicationScope<TE, EE, T, E> {
    pub fn new(transport: T ,external_client: E) -> ApplicationScope<TE, EE, T, E> {
        let command_executor = Rc::new(RefCell::new(Executor::new()));
        let arc_client = Arc::new(Mutex::new(Client::new(transport)));
        let arc_external_client = Arc::new(external_client);
        let app_state = Rc::new(RefCell::new(AppState::new()));

        let tree = Tree::new(vec![]);
        let left_panel_tree = Tree::new(vec![LeftPanelTabKind::Layers, LeftPanelTabKind::Components]);
        let right_panel_tree = Tree::new(vec![RightPanelTabKind::Tool, RightPanelTabKind::Appearance, RightPanelTabKind::Properties]);

        ApplicationScope {
            command_executor,
            state: app_state,
            client: arc_client,
            external_client: arc_external_client,
            _phantom: PhantomData,
            projects_tree: Rc::new(RefCell::new(tree)),
            left_panel_tree: Rc::new(RefCell::new(left_panel_tree)),
            right_panel_tree: Rc::new(RefCell::new(right_panel_tree))
        }
    }

    /// Getter for a non mutable reference to application wide state
    pub fn state(&self) -> Ref<AppState> {
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
    pub fn external_client(&self) -> Arc<E> {
        self.external_client.clone()
    }
    
    /// Getting a reference for the client
    pub fn client(&self) -> Arc<Mutex<Client<TE,T>>> {
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
        self.state_mut().add_project(id, title, zoom, screens);
        let count = self.state().tab_count();
        self.projects_tree.borrow_mut().push_to_first_leaf(count - 1);
    }

    pub fn projects_tree(&self) -> RefMut<'_, Tree<usize>> {
        self.projects_tree.borrow_mut()
    }

    pub fn left_panel_tree(&self) -> RefMut<'_, Tree<LeftPanelTabKind>> {
        self.left_panel_tree.borrow_mut()
    }

    pub fn right_panel_tree(&self) -> RefMut<'_, Tree<RightPanelTabKind>> {
        self.right_panel_tree.borrow_mut()
    }
}

pub struct CreateProjectWindowScope <TE: Debug + Send, T: ClientTransport<TE> > {
    client: Arc<Mutex<Client<TE, T>>>,
    state: Rc<RefCell<CreateProjectWindowState>>
}

impl <TE: Debug + Send, T: ClientTransport<TE>> CreateProjectWindowScope<TE, T> {
    pub fn new(client: Arc<Mutex<Client<TE, T>>> ) -> CreateProjectWindowScope<TE, T> {
        CreateProjectWindowScope { client , state: Rc::new(RefCell::new( CreateProjectWindowState::new())) }
    }

    /// Getter for a non mutable reference to application wide state
    pub fn state(&self) -> Ref<CreateProjectWindowState> {
        self.state.borrow()
    }

    /// Getter for a mutable reference to application wide state
    pub fn state_mut(&self) -> RefMut<CreateProjectWindowState> {
        self.state.borrow_mut()
    }

}

pub struct TabScope <TE: Debug + Send, T: ClientTransport<TE>> {
    client: Arc<Mutex<Client<TE, T>>>,
    state: Rc<RefCell<TabInfo>>,
}

impl <TE: Debug + Send, T: ClientTransport<TE>> TabScope<TE, T> {
    pub fn new(client: Arc<Mutex<Client<TE, T>>>, state: Rc<RefCell<TabInfo>>) -> TabScope<TE, T> {
        TabScope { client, state }
    }
}
