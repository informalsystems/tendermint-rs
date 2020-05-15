use crate::prelude::*;

use std::collections::btree_map::Entry::*;
use std::collections::BTreeMap;

use VerifiedStatus::*;

#[derive(Debug)]
pub struct MemoryStore {
    store: BTreeMap<Height, StoreEntry>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            store: BTreeMap::new(),
        }
    }
}

impl LightStore for MemoryStore {
    fn get_any(&self, height: Height) -> Option<LightBlock> {
        self.store.get(&height).cloned().map(|e| e.light_block)
    }

    fn get_verified(&self, height: Height) -> Option<LightBlock> {
        self.store
            .get(&height)
            .filter(|e| e.status == Verified)
            .cloned()
            .map(|e| e.light_block)
    }

    fn insert_verified(&mut self, light_block: LightBlock) {
        self.store
            .insert(light_block.height(), StoreEntry::new(light_block, Verified));
    }

    fn remove_verified(&mut self, height: Height) {
        if let Occupied(e) = self.store.entry(height) {
            if e.get().status == Verified {
                e.remove();
            }
        }
    }

    fn latest_verified(&self) -> Option<LightBlock> {
        self.store
            .iter()
            .rev()
            .find(|(_, e)| e.status == Verified)
            .map(|(_, e)| e.light_block.clone())
    }

    fn all_verified(&self) -> Vec<LightBlock> {
        self.store
            .iter()
            .filter(|(_, e)| e.status == Verified)
            .map(|(_, e)| e.light_block.clone())
            .collect()
    }

    fn get_unverified(&self, height: Height) -> Option<LightBlock> {
        self.store
            .get(&height)
            .filter(|e| e.status == Unverified)
            .cloned()
            .map(|e| e.light_block)
    }

    fn insert_unverified(&mut self, light_block: LightBlock) {
        self.store.insert(
            light_block.height(),
            StoreEntry::new(light_block, Unverified),
        );
    }

    fn remove_unverified(&mut self, height: Height) {
        if let Occupied(e) = self.store.entry(height) {
            if e.get().status == Unverified {
                e.remove();
            }
        }
    }

    fn latest_unverified(&self) -> Option<LightBlock> {
        self.store
            .iter()
            .rev()
            .find(|(_, e)| e.status == Unverified)
            .map(|(_, e)| e.light_block.clone())
    }

    fn all_unverified(&self) -> Vec<LightBlock> {
        self.store
            .iter()
            .filter(|(_, e)| e.status == Unverified)
            .map(|(_, e)| e.light_block.clone())
            .collect()
    }

    fn get_failed(&self, height: Height) -> Option<LightBlock> {
        self.store
            .get(&height)
            .filter(|e| e.status == Failed)
            .cloned()
            .map(|e| e.light_block)
    }

    fn insert_failed(&mut self, light_block: LightBlock) {
        self.store
            .insert(light_block.height(), StoreEntry::new(light_block, Failed));
    }

    fn remove_failed(&mut self, height: Height) {
        if let Occupied(e) = self.store.entry(height) {
            if e.get().status == Failed {
                e.remove();
            }
        }
    }

    fn latest_failed(&self) -> Option<LightBlock> {
        self.store
            .iter()
            .rev()
            .find(|(_, e)| e.status == Failed)
            .map(|(_, e)| e.light_block.clone())
    }

    fn all_failed(&self) -> Vec<LightBlock> {
        self.store
            .iter()
            .filter(|(_, e)| e.status == Failed)
            .map(|(_, e)| e.light_block.clone())
            .collect()
    }
}
