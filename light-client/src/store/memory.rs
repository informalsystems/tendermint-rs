use crate::{
    store::{LightStore, Status},
    types::{Height, LightBlock},
};

use std::collections::btree_map::Entry::*;
use std::collections::BTreeMap;

/// Internal entry for the memory store
#[derive(Clone, Debug, PartialEq)]
struct StoreEntry<LB> {
    light_block: LB,
    status: Status,
}

impl<LB> StoreEntry<LB> {
    fn new(light_block: LB, status: Status) -> Self {
        Self {
            light_block,
            status,
        }
    }
}

/// Transient in-memory store.
#[derive(Debug, Clone, Default)]
pub struct MemoryStore<LB> {
    store: BTreeMap<Height, StoreEntry<LB>>,
}

impl<LB> MemoryStore<LB> {
    pub fn new() -> Self {
        Self {
            store: BTreeMap::new(),
        }
    }
}

impl<LB> LightStore<LB> for MemoryStore<LB>
where
    LB: LightBlock,
{
    fn get(&self, height: Height, status: Status) -> Option<LB> {
        self.store
            .get(&height)
            .filter(|e| e.status == status)
            .cloned()
            .map(|e| e.light_block)
    }

    fn insert(&mut self, light_block: LB, status: Status) {
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

    fn update(&mut self, light_block: &LB, status: Status) {
        self.insert(light_block.clone(), status);
    }

    fn latest(&self, status: Status) -> Option<LB> {
        self.store
            .iter()
            .filter(|(_, e)| e.status == status)
            .max_by_key(|(&height, _)| height)
            .map(|(_, e)| e.light_block.clone())
    }

    fn all(&self, status: Status) -> Box<dyn Iterator<Item = LB>> {
        let light_blocks: Vec<_> = self
            .store
            .iter()
            .filter(|(_, e)| e.status == status)
            .map(|(_, e)| e.light_block.clone())
            .collect();

        Box::new(light_blocks.into_iter())
    }
}
