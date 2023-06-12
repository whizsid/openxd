//! Application Scopes
//!
//! Some shared values/interfaces only required for a specific scope (for canvas, for menu bar).
//! But some values/interfaces are application wide. So we have to redefine what are the required
//! parameters for each component. Those scopes will avoid those redefinitions.
use std::{fmt::Debug, marker::PhantomData, sync::Arc, rc::Rc, cell::{RefCell, Ref, RefMut}};

use futures::lock::Mutex;

use crate::{client::{ClientTransport, Client}, external::External, commands::{Executor, Command}, state::{AppState, CreateProjectWindowState}};

/// Application wide scope
pub struct ApplicationScope<TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>> {
    client: Arc<Mutex<Client<TE, T>>>,
    external_client: Arc<E>,
    command_executor: Rc<RefCell<Executor>>,
    state: Rc<RefCell<AppState>>,
    _phantom: PhantomData<TE>
}

impl <TE: Debug + Send, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>> ApplicationScope<TE, EE, T, E> {
    pub fn new(transport: T ,external_client: E) -> ApplicationScope<TE, EE, T, E> {
        let command_executor = Rc::new(RefCell::new(Executor::new()));
        let arc_client = Arc::new(Mutex::new(Client::new(transport)));
        let arc_external_client = Arc::new(external_client);
        let app_state = Rc::new(RefCell::new(AppState::new()));

        ApplicationScope {
            command_executor,
            state: app_state,
            client: arc_client,
            external_client: arc_external_client,
            _phantom: PhantomData
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
