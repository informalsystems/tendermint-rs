//! Interface and implementations of the light block store.
//!
//! See the `memory` and `sled` modules for:
//!
//! - a transient, in-memory implementation for testing purposes
//! - a persistent, on-disk, sled-backed implementation for production

use core::fmt::Debug;

use crate::prelude::*;
use crate::types::{Height, LightBlock, Status};
use crate::utils::std_ext;

pub mod memory;

#[cfg(feature = "lightstore-sled")]
#[cfg_attr(docsrs, doc(cfg(feature = "lightstore-sled")))]
pub mod sled;

/// Store for light blocks.
///
/// The light store records light blocks received from peers, and their verification status.
/// Additionally, the light store will contain one or more trusted light blocks specified
/// at initialization time.
///
/// ## Implements
/// - [LCV-DIST-STORE.1]
pub trait LightStore: Debug + Send + Sync {
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
    fn highest(&self, status: Status) -> Option<LightBlock>;

    /// Get the light block of lowest height with the given status.
    fn lowest(&self, status: Status) -> Option<LightBlock>;

    /// Get an iterator of all light blocks with the given status.
    fn all(&self, status: Status) -> Box<dyn Iterator<Item = LightBlock>>;

    /// Get a block at a given height whatever its verification status as long as it hasn't failed
    /// verification (ie. its status is not `Status::Failed`).
    fn get_non_failed(&self, height: Height) -> Option<(LightBlock, Status)> {
        None.or_else(|| {
            self.get(height, Status::Trusted)
                .map(|lb| (lb, Status::Trusted))
        })
        .or_else(|| {
            self.get(height, Status::Verified)
                .map(|lb| (lb, Status::Verified))
        })
        .or_else(|| {
            self.get(height, Status::Unverified)
                .map(|lb| (lb, Status::Unverified))
        })
    }

    /// Get the light block of greatest height with the trusted or verified status.
    fn highest_trusted_or_verified(&self) -> Option<LightBlock> {
        let latest_trusted = self.highest(Status::Trusted);
        let latest_verified = self.highest(Status::Verified);

        std_ext::option::select(latest_trusted, latest_verified, |t, v| {
            std_ext::cmp::max_by_key(t, v, |lb| lb.height())
        })
    }

    /// Get the light block of lowest height with the trusted or verified status.
    fn lowest_trusted_or_verified(&self) -> Option<LightBlock> {
        let lowest_trusted = self.lowest(Status::Trusted);
        let lowest_verified = self.lowest(Status::Verified);

        std_ext::option::select(lowest_trusted, lowest_verified, |t, v| {
            std_ext::cmp::min_by_key(t, v, |lb| lb.height())
        })
    }

    /// Get the light block of the given height with the trusted or verified status.
    fn get_trusted_or_verified(&self, height: Height) -> Option<LightBlock> {
        self.get(height, Status::Trusted)
            .or_else(|| self.get(height, Status::Verified))
    }
}
