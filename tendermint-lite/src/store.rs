use tendermint::lite::{Error, Header, Height, Store, TrustedState};

use std::collections::HashMap;
use tendermint::block;

pub type State = TrustedState<block::signed_header::SignedHeader, block::header::Header>;

#[derive(Default)]
pub struct MemStore {
    height: Height,
    store: HashMap<Height, State>,
}

impl MemStore {
    pub fn new() -> MemStore {
        MemStore {
            height: 0,
            store: HashMap::new(),
        }
    }
}

impl Store<block::signed_header::SignedHeader, block::header::Header> for MemStore {
    fn add(&mut self, trusted: State) -> Result<(), Error> {
        let height = trusted.last_header().header().height();
        self.height = height;
        self.store.insert(height, trusted);
        Ok(())
    }

    fn get(&self, h: Height) -> Result<&State, Error> {
        let mut height = h;
        if h == 0 {
            height = self.height
        }
        match self.store.get(&height) {
            Some(state) => Ok(state),
            None => Err(Error::RequestFailed),
        }
    }
}
