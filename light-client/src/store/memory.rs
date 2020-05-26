use crate::prelude::*;

use std::collections::btree_map::Entry::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct StoreEntry {
    light_block: LightBlock,
    status: VerifiedStatus,
}

impl StoreEntry {
    pub fn new(light_block: LightBlock, status: VerifiedStatus) -> Self {
        Self {
            light_block,
            status,
        }
    }
}

#[derive(Debug, Default)]
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
    fn get(&self, height: Height, status: VerifiedStatus) -> Option<LightBlock> {
        self.store
            .get(&height)
            .filter(|e| e.status == status)
            .cloned()
            .map(|e| e.light_block)
    }

    fn insert(&mut self, light_block: LightBlock, status: VerifiedStatus) {
        self.store
            .insert(light_block.height(), StoreEntry::new(light_block, status));
    }

    fn remove(&mut self, height: Height, status: VerifiedStatus) {
        if let Occupied(e) = self.store.entry(height) {
            if e.get().status == status {
                e.remove_entry();
            }
        }
    }

    fn update(&mut self, light_block: LightBlock, status: VerifiedStatus) {
        self.insert(light_block, status);
    }

    fn latest(&self, status: VerifiedStatus) -> Option<LightBlock> {
        self.store
            .iter()
            .rev()
            .find(|(_, e)| e.status == status)
            .map(|(_, e)| e.light_block.clone())
    }

    fn all(&self, status: VerifiedStatus) -> Box<dyn Iterator<Item = LightBlock>> {
        let light_blocks: Vec<_> = self
            .store
            .iter()
            .filter(|(_, e)| e.status == status)
            .map(|(_, e)| e.light_block.clone())
            .collect();

        Box::new(light_blocks.into_iter())
    }
}
