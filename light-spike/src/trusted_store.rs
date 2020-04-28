use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::prelude::*;

#[derive(Debug, Default)]
pub struct TrustedStore {
    store: HashMap<Height, TrustedState>,
}

impl TrustedStore {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn split(self) -> (TSReader, TSReadWriter) {
        let store = Arc::new(RwLock::new(self));
        let reader = TSReader { ts: store.clone() };
        let writer = TSReadWriter { ts: store };

        (reader, writer)
    }
}

impl TrustedStore {
    pub fn get(&self, height: Height) -> Option<&TrustedState> {
        self.store.get(&height)
    }

    pub fn add(&mut self, trusted_state: TrustedState) {
        self.store
            .insert(trusted_state.header.height, trusted_state);
    }
}

#[derive(Clone, Debug)]
pub struct TSReader {
    ts: Arc<RwLock<TrustedStore>>,
}

impl TSReader {
    pub fn get(&self, height: Height) -> Option<TrustedState> {
        self.ts.read().unwrap().get(height).cloned()
    }
}

#[derive(Debug)]
pub struct TSReadWriter {
    ts: Arc<RwLock<TrustedStore>>,
}

impl TSReadWriter {
    pub fn get(&self, height: Height) -> Option<TrustedState> {
        self.ts.read().unwrap().get(height).cloned()
    }

    pub fn add(&mut self, trusted_state: TrustedState) {
        let mut ts = self.ts.write().unwrap();
        ts.add(trusted_state);
    }
}
