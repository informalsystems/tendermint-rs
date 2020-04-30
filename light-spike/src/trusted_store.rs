use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
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
        let store = Arc::new(Mutex::new(self));
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
        self.store.insert(trusted_state.height, trusted_state);
    }

    pub fn all(&self) -> Vec<&TrustedState> {
        self.store.values().collect()
    }
}

#[derive(Clone, Debug)]
pub struct TSReader {
    ts: Arc<Mutex<TrustedStore>>,
}

impl TSReader {
    pub fn get(&self, height: Height) -> Option<TrustedState> {
        self.ts.lock().unwrap().get(height).cloned()
    }

    pub fn all(&self) -> Vec<TrustedState> {
        self.ts
            .lock()
            .unwrap()
            .all()
            .into_iter()
            .map(Clone::clone)
            .collect()
    }
}

#[derive(Debug)]
pub struct TSReadWriter {
    ts: Arc<Mutex<TrustedStore>>,
}

impl TSReadWriter {
    pub fn get(&self, height: Height) -> Option<TrustedState> {
        self.ts.lock().unwrap().get(height).cloned()
    }

    pub fn add(&self, trusted_state: TrustedState) {
        let mut ts = self.ts.lock().unwrap();
        ts.add(trusted_state);
    }
}
