use crate::state::State;
use tendermint::block::Height;
use tendermint::lite::{Error, Header, SignedHeader, Store, TrustedState};

use std::collections::HashMap;

#[derive(Default)]
pub struct MemStore {
    height: Height,
    store: HashMap<Height, State>,
}

impl MemStore {
    pub fn new() -> MemStore {
        MemStore {
            height: Height::from(0),
            store: HashMap::new(),
        }
    }
}

impl Store for MemStore {
    type TrustedState = State;

    fn add(&mut self, trusted: &Self::TrustedState) -> Result<(), Error> {
        let height = trusted.last_header().header().height();
        self.height = height;
        self.store.insert(height, trusted.clone());
        Ok(())
    }

    fn get(&self, h: Height) -> Result<&Self::TrustedState, Error> {
        let mut height = h;
        if h.value() == 0 {
            height = self.height
        }
        match self.store.get(&height) {
            Some(state) => Ok(state),
            None => Err(Error::RequestFailed),
        }
    }
}
