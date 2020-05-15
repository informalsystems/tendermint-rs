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
    fn get_any(&self, height: Height) -> Option<LightBlock>;

    fn get_verified(&self, height: Height) -> Option<LightBlock>;
    fn insert_verified(&mut self, light_block: LightBlock);
    fn remove_verified(&mut self, height: Height);
    fn latest_verified(&self) -> Option<LightBlock>;
    fn all_verified(&self) -> Vec<LightBlock>;

    fn get_unverified(&self, height: Height) -> Option<LightBlock>;
    fn insert_unverified(&mut self, light_block: LightBlock);
    fn remove_unverified(&mut self, height: Height);
    fn latest_unverified(&self) -> Option<LightBlock>;
    fn all_unverified(&self) -> Vec<LightBlock>;

    fn get_failed(&self, height: Height) -> Option<LightBlock>;
    fn insert_failed(&mut self, light_block: LightBlock);
    fn remove_failed(&mut self, height: Height);
    fn latest_failed(&self) -> Option<LightBlock>;
    fn all_failed(&self) -> Vec<LightBlock>;
}

