//! Application Scopes
//!
//! Some shared values/interfaces only required for a specific scope (for canvas, for menu bar).
//! But some values/interfaces are application wide. So we have to redefine what are the required
//! parameters for each component. Those scopes will avoid those redefinitions.
use std::{fmt::Debug, marker::PhantomData, sync::Arc, rc::Rc, cell::{RefCell, Ref, RefMut}};

use futures::lock::Mutex;

use crate::{client::{ClientTransport, Client}, remote_cache::RemoteCache, commands::{Executor, Command}, state::AppState};

/// Application wide scope
pub struct ApplicationScope<TE: Debug + Send, CE: Debug, T: ClientTransport<TE>, C: RemoteCache<Error = CE>> {
    client: Arc<Mutex<Client<TE, T>>>,
    remote_cache: Arc<C>,
    command_executor: Rc<RefCell<Executor>>,
    state: Rc<RefCell<AppState>>,
    _phantom: PhantomData<TE>
}

impl <TE: Debug + Send, CE: Debug, T: ClientTransport<TE>, C: RemoteCache<Error = CE>> ApplicationScope<TE, CE, T, C> {
    pub fn new(transport: T ,remote_cache: C) -> ApplicationScope<TE, CE, T, C> {
        let command_executor = Rc::new(RefCell::new(Executor::new()));
        let arc_client = Arc::new(Mutex::new(Client::new(transport)));
        let arc_remote_cache = Arc::new(remote_cache);
        let app_state = Rc::new(RefCell::new(AppState::new()));

        ApplicationScope {
            command_executor,
            state: app_state,
            client: arc_client,
            remote_cache: arc_remote_cache,
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

    /// Getting a reference for the remote cache
    pub fn remote_cache(&self) -> Arc<C> {
        self.remote_cache.clone()
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
