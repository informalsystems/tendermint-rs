//! Interface and implementations of the light block store.
//!
//! See the `memory` and `sled` modules for:
//! - a transient, in-memory implementation for testing purposes
//! - a persistent, on-disk, sled-backed implementation for production

use crate::types::{Height, LightBlock, Status};

pub mod memory;
pub mod sled;

/// Store for light blocks.
///
/// The light store records light blocks received from peers, and their verification status.
/// Additionally, the light store will contain one or more trusted light blocks specified
/// at initialization time.
///
/// ## Implements
/// - [LCV-DIST-STORE.1]
pub trait LightStore: std::fmt::Debug + Send {
    /// Get the light block at the given height with the given status, or return `None` otherwise.
    fn get(&self, height: Height, status: Status) -> Option<LightBlock>;
    /// Update the `status` of the given `light_block`.
    fn update(&mut self, light_block: &LightBlock, status: Status);
    /// Insert a new light block in the store with the given status.
    /// Overrides any other block with the same height and status.
    fn insert(&mut self, light_block: LightBlock, status: Status);
    /// Remove the light block with the given height and status, if any.
    fn remove(&mut self, height: Height, status: Status);
    /// Get the highest light block with the given status.
    fn highest(&self, status: Status) -> Option<LightBlock>;
    /// Get an iterator of all light blocks with the given status.
    fn all(&self, status: Status) -> Box<dyn Iterator<Item = LightBlock>>;
}
