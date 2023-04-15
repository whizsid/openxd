use std::fmt::Debug;

use futures::{Sink, Stream};

use crate::{client::Client, state::AppState};

pub struct App<E: Debug, T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin> {
    state: AppState,
    client: Client<E, T>,
}

impl<E: Debug, T: Stream<Item = Vec<u8>> + Sink<Vec<u8>, Error = E> + Unpin> App<E, T> {
    pub fn new(internal_transport: T) -> App<E, T> {
        Self {
            state: AppState::new(),
            client: Client::new(internal_transport),
        }
    }

    pub fn file_dialog_opened(&mut self) {
        self.state.disable_main_ui();
    }

    pub fn file_dialog_done(&mut self, _buf: Vec<u8>) {
        self.state.enable_main_ui();
    }

    pub fn file_dialog_canceled(&mut self) {
        self.state.enable_main_ui();
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }
}
