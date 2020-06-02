//! Interface and implementations of the light block store.
//!
//! See the `memory` and `sled` modules for:
//! - a transient, in-memory implementation for testing purposes
//! - a persistent, on-disk, sled-backed implementation for production

use crate::prelude::*;

use serde::{Deserialize, Serialize};

pub mod memory;
pub mod sled;

/// Verification status of a light block.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerifiedStatus {
    /// The light has not been verified yet.
    Unverified,
    /// The light block has been successfully verified.
    Verified,
    /// The light block has failed verification.
    Failed,
}

impl VerifiedStatus {
    /// Return a slice of all the possible values for this enum.
    pub fn iter() -> &'static [VerifiedStatus] {
        static ALL: &[VerifiedStatus] = &[
            VerifiedStatus::Unverified,
            VerifiedStatus::Verified,
            VerifiedStatus::Failed,
        ];

        ALL
    }
}

/// Store for light blocks.
///
/// The light store records light blocks received from peers, and their verification status.
/// Additionally, the light store will contain one or more trusted light blocks specified
/// at initialization time.
///
/// ## Implements
/// - [LCV-DIST-STORE.1]
pub trait LightStore: std::fmt::Debug {
    /// Get the light block at the given height with the given status, or return `None` otherwise.
    fn get(&self, height: Height, status: VerifiedStatus) -> Option<LightBlock>;
    /// Update the `status` of the given `light_block`.
    fn update(&mut self, light_block: LightBlock, status: VerifiedStatus);
    /// Insert a new light block in the store with the given status.
    /// Overrides any other block with the same height and status.
    fn insert(&mut self, light_block: LightBlock, status: VerifiedStatus);
    /// Remove the light block with the given height and status, if any.
    fn remove(&mut self, height: Height, status: VerifiedStatus);
    /// Get the highest light block with the given status.
    fn latest(&self, status: VerifiedStatus) -> Option<LightBlock>;
    /// Get an iterator of all light blocks with the given status.
    fn all(&self, status: VerifiedStatus) -> Box<dyn Iterator<Item = LightBlock>>;
}
