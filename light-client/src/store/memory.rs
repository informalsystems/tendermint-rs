//! Transient in-memory store

use crate::store::{LightStore, Status};
use crate::verifier::types::{Height, LightBlock};

use std::collections::btree_map::Entry::*;
use std::collections::BTreeMap;

/// Internal entry for the memory store
#[derive(Clone, Debug, PartialEq)]
struct StoreEntry {
    light_block: LightBlock,
    status: Status,
}

impl StoreEntry {
    fn new(light_block: LightBlock, status: Status) -> Self {
        Self {
            light_block,
            status,
        }
    }
}

/// Transient in-memory store.
#[derive(Debug, Clone, Default)]
pub struct MemoryStore {
    store: BTreeMap<Height, StoreEntry>,
}

impl MemoryStore {
    /// Create a new, empty, in-memory store
    pub fn new() -> Self {
        Self {
            store: BTreeMap::new(),
        }
    }
}

impl LightStore for MemoryStore {
    fn get(&self, height: Height, status: Status) -> Option<LightBlock> {
        self.store
            .get(&height)
            .filter(|e| e.status == status)
            .cloned()
            .map(|e| e.light_block)
    }

    fn insert(&mut self, light_block: LightBlock, status: Status) {
        self.store
            .insert(light_block.height(), StoreEntry::new(light_block, status));
    }

    fn remove(&mut self, height: Height, status: Status) {
        if let Occupied(e) = self.store.entry(height) {
            if e.get().status == status {
                e.remove_entry();
            }
        }
    }

    fn update(&mut self, light_block: &LightBlock, status: Status) {
        self.insert(light_block.clone(), status);
    }

    fn highest(&self, status: Status) -> Option<LightBlock> {
        self.store
            .iter()
            .filter(|(_, e)| e.status == status)
            .max_by_key(|(&height, _)| height)
            .map(|(_, e)| e.light_block.clone())
    }

    fn lowest(&self, status: Status) -> Option<LightBlock> {
        self.store
            .iter()
            .filter(|(_, e)| e.status == status)
            .min_by_key(|(&height, _)| height)
            .map(|(_, e)| e.light_block.clone())
    }

    #[allow(clippy::needless_collect)]
    fn all(&self, status: Status) -> Box<dyn Iterator<Item = LightBlock>> {
        let light_blocks: Vec<_> = self
            .store
            .iter()
            .filter(|(_, e)| e.status == status)
            .map(|(_, e)| e.light_block.clone())
            .collect();

        Box::new(light_blocks.into_iter())
    }
}
