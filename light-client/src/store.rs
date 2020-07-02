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

    /// Get the light block of greatest height with the given status.
    fn latest(&self, status: Status) -> Option<LightBlock>;

    /// Get an iterator of all light blocks with the given status.
    fn all(&self, status: Status) -> Box<dyn Iterator<Item = LightBlock>>;

    /// Get the light block of greatest height with the trusted or verified status.
    fn latest_trusted_or_verified(&self) -> Option<LightBlock> {
        let latest_trusted = self.latest(Status::Trusted);
        let latest_verified = self.latest(Status::Verified);

        match (latest_trusted, latest_verified) {
            (None, latest_verified) => latest_verified,
            (latest_trusted, None) => latest_trusted,
            (Some(latest_trusted), Some(latest_verified)) => {
                if latest_trusted.height() > latest_verified.height() {
                    Some(latest_trusted)
                } else {
                    Some(latest_verified)
                }
            }
        }
    }
}
