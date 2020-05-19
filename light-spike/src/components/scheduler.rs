use crate::prelude::*;

#[contract_trait]
pub trait Scheduler {
    #[pre(light_store.latest(VerifiedStatus::Verified).is_some())]
    #[post(valid_schedule(ret, target_height, next_height, light_store))]
    fn schedule(
        &self,
        light_store: &dyn LightStore,
        next_height: Height,
        target_height: Height,
    ) -> Height;
}

#[contract_trait]
impl<F> Scheduler for F
where
    F: Fn(&dyn LightStore, Height, Height) -> Height,
{
    fn schedule(
        &self,
        light_store: &dyn LightStore,
        next_height: Height,
        target_height: Height,
    ) -> Height {
        self(light_store, next_height, target_height)
    }
}

#[pre(light_store.latest(VerifiedStatus::Verified).is_some())]
#[post(valid_schedule(ret, target_height, next_height, light_store))]
pub fn schedule(
    light_store: &dyn LightStore,
    next_height: Height,
    target_height: Height,
) -> Height {
    let latest_trusted_height = light_store
        .latest(VerifiedStatus::Verified)
        .map(|lb| lb.height())
        .unwrap();

    if latest_trusted_height == next_height && latest_trusted_height < target_height {
        target_height
    } else if latest_trusted_height < next_height && latest_trusted_height < target_height {
        midpoint(latest_trusted_height, next_height)
    } else if latest_trusted_height == target_height {
        target_height
    } else {
        midpoint(next_height, target_height)
    }
}

fn valid_schedule(
    scheduled_height: Height,
    target_height: Height,
    next_height: Height,
    light_store: &dyn LightStore,
) -> bool {
    let latest_trusted_height = light_store
        .latest(VerifiedStatus::Verified)
        .map(|lb| lb.height())
        .unwrap();

    if latest_trusted_height == next_height && latest_trusted_height < target_height {
        next_height < scheduled_height && scheduled_height <= target_height
    } else if latest_trusted_height < next_height && latest_trusted_height < target_height {
        latest_trusted_height < scheduled_height && scheduled_height < next_height
    } else if latest_trusted_height == target_height {
        scheduled_height == target_height
    } else {
        true
    }
}

#[pre(low < high)]
#[post(low < ret && ret <= high)]
fn midpoint(low: Height, high: Height) -> Height {
    low + (high + 1 - low) / 2
}
