// TODO: Replace this in-memory store with a proper `sled` based implementation

use crate::prelude::*;

pub mod memory;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VerifiedStatus {
    Unverified,
    Verified,
    Failed,
}

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

pub trait LightStore: std::fmt::Debug {
    fn get(&self, height: Height, status: VerifiedStatus) -> Option<LightBlock>;
    fn insert(&mut self, light_block: LightBlock, status: VerifiedStatus);
    fn remove(&mut self, height: Height, status: VerifiedStatus);
    fn latest(&self, status: VerifiedStatus) -> Option<LightBlock>;
    fn all(&self, status: VerifiedStatus) -> Vec<LightBlock>;
}

