use tendermint::lite::{Header, Height, TrustedState};

use std::collections::HashMap;
use tendermint::block;
use tendermint::lite::error::{Error, Kind};

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

impl MemStore {
    pub fn add(&mut self, trusted: State) -> Result<(), Error> {
        let height = trusted.last_header().header().height();
        self.height = height;
        self.store.insert(height, trusted);
        Ok(())
    }

    pub fn get(&self, h: Height) -> Result<&State, Error> {
        let mut height = h;
        if h == 0 {
            height = self.height
        }
        match self.store.get(&height) {
            Some(state) => Ok(state),
            None => Err(Kind::RequestFailed
                .context(format!("could not load height {} from store", height))
                .into()),
        }
    }
}
