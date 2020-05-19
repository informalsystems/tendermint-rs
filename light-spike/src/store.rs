use crate::prelude::*;

use serde::{Deserialize, Serialize};

pub mod memory;
pub mod sled;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerifiedStatus {
    Unverified,
    Verified,
    Failed,
}

pub trait LightStore: std::fmt::Debug {
    fn get(&self, height: Height, status: VerifiedStatus) -> Option<LightBlock>;
    fn update(&mut self, light_block: LightBlock, status: VerifiedStatus);
    fn insert(&mut self, light_block: LightBlock, status: VerifiedStatus);
    fn remove(&mut self, height: Height, status: VerifiedStatus);
    fn latest(&self, status: VerifiedStatus) -> Option<LightBlock>;
    fn all(&self, status: VerifiedStatus) -> Box<dyn Iterator<Item = LightBlock>>;
}
