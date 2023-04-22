use std::{fmt::Debug, marker::PhantomData, sync::Arc, rc::Rc, cell::{RefCell, Ref, RefMut}};

use futures::lock::Mutex;

use crate::{client::{ClientTransport, Client}, remote_cache::RemoteCache, commands::{Executor, Command}, state::AppState};

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

    pub fn state(&self) -> Ref<AppState> {
        self.state.borrow()
    }

    pub fn state_mut(&self) -> RefMut<AppState> {
        self.state.borrow_mut()
    }

    pub fn execute<CMD: Command + 'static>(&self, cmd: CMD) {
        self.command_executor.borrow_mut().execute(cmd);
    }

    pub fn remote_cache(&self) -> Arc<C> {
        self.remote_cache.clone()
    }
    
    pub fn client(&self) -> Arc<Mutex<Client<TE,T>>> {
        self.client.clone()
    }

    pub fn update_cmd_executor(&self) {
        self.command_executor.borrow_mut().update();
    }
}
