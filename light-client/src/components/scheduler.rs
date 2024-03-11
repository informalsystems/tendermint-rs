//! Provides an interface and default implementation of the `Scheduler` component

use contracts::*;

use crate::{store::LightStore, verifier::types::Height};

/// The scheduler decides what block to verify next given the current and target heights.
///
/// The scheduler is given access to the light store, in order to optionally
/// improve performance by picking a next block that has already been fetched.
#[contract_trait]
#[allow(missing_docs)] // This is required because of the `contracts` crate (TODO: open/link issue)
pub trait Scheduler: Send + Sync {
    /// Decides what block to verify next.
    ///
    /// ## Precondition
    /// - The light store contains at least one verified block. [LCV-SCHEDULE-PRE.1]
    ///
    /// ## Postcondition
    /// - The resulting height must be valid according to `valid_schedule`. [LCV-SCHEDULE-POST.1]
    #[requires(light_store.highest_trusted_or_verified_before(target_height).is_some())]
    #[ensures(valid_schedule(ret, target_height, current_height, light_store))]
    fn schedule(
        &self,
        light_store: &dyn LightStore,
        current_height: Height,
        target_height: Height,
    ) -> Height;
}

#[contract_trait]
impl<F: Send + Sync> Scheduler for F
where
    F: Fn(&dyn LightStore, Height, Height) -> Height,
{
    fn schedule(
        &self,
        light_store: &dyn LightStore,
        current_height: Height,
        target_height: Height,
    ) -> Height {
        self(light_store, current_height, target_height)
    }
}

/// Basic bisecting scheduler which picks the appropriate midpoint without
/// optimizing for performance using the blocks available in the light store.
///
/// ## Precondition
/// - The light store contains at least one verified block. [LCV-SCHEDULE-PRE.1]
///
/// ## Postcondition
/// - The resulting height must be valid according to `valid_schedule`. [LCV-SCHEDULE-POST.1]
#[requires(light_store.highest_trusted_or_verified().is_some())]
#[ensures(valid_schedule(ret, target_height, current_height, light_store))]
pub fn basic_bisecting_schedule(
    light_store: &dyn LightStore,
    current_height: Height,
    target_height: Height,
) -> Height {
    let trusted_height = light_store
        .highest_trusted_or_verified_before(target_height)
        .map(|lb| lb.height())
        .unwrap();

    if trusted_height == current_height {
        // We can't go further back, so let's try to verify the target height again,
        // hopefully we have enough trust in the store by now.
        target_height
    } else {
        // Pick a midpoint H between `trusted_height <= H <= current_height`.
        midpoint(trusted_height, current_height)
    }
}

/// Checks whether the given `scheduled_height` is a valid schedule according to the
/// following specification.
///
/// - i) If `latest_verified_height == current_height` and `latest_verified_height < target_height`
/// then `current_height < scheduled_height <= target_height`.
///
/// - ii) If `latest_verified_height < current_height` and `latest_verified_height < target_height`
/// then `latest_verified_height < scheduled_height < current_height`.
///
/// - iii) If `latest_verified_height = target_height` then `scheduled_height == target_height`.
///
/// ## Note
///
/// - Case i. captures the case where the light block at height `current_height` has been verified,
///   and we can choose a height closer to the `target_height`. As we get the `light_store` as
///   parameter, the choice of the next height can depend on the `light_store`, e.g., we can pick a
///   height for which we have already downloaded a light block.
/// - In Case ii. the header at `current_height` could not be verified, and we need to pick a lesser
///   height.
/// - In Case iii. is a special case when we have verified the `target_height`.
///
/// ## Implements
/// - [LCV-SCHEDULE-POST.1]
pub fn valid_schedule(
    scheduled_height: Height,
    target_height: Height,
    current_height: Height,
    light_store: &dyn LightStore,
) -> bool {
    let latest_trusted_height = light_store
        .highest_trusted_or_verified_before(target_height)
        .map(|lb| lb.height())
        .unwrap();

    if latest_trusted_height == current_height && latest_trusted_height < target_height {
        current_height < scheduled_height && scheduled_height <= target_height
    } else if latest_trusted_height < current_height && latest_trusted_height < target_height {
        latest_trusted_height < scheduled_height && scheduled_height < current_height
    } else if latest_trusted_height == target_height {
        scheduled_height == target_height
    } else {
        true
    }
}

#[requires(low <= high)]
#[ensures(low <= ret && ret <= high)]
fn midpoint(low: Height, high: Height) -> Height {
    (low.value() + (high.value() + 1 - low.value()) / 2)
        .try_into()
        .unwrap() // Will panic if midpoint is higher than i64::MAX
}
