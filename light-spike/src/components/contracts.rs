use crate::prelude::*;

pub fn trusted_state_contains_block_within_trusting_period(
    light_store: &dyn LightStore,
    trusting_period: Duration,
    now: Time,
) -> bool {
    light_store
        .all_verified()
        .iter()
        .any(|lb| is_within_trust_period(lb, trusting_period, now))
}

pub fn target_height_greater_than_all_blocks_in_trusted_store(
    light_store: &dyn LightStore,
    target_height: Height,
) -> bool {
    light_store
        .all_verified()
        .iter()
        .all(|lb| lb.height() < target_height)
}

pub fn trusted_store_contains_block_at_target_height(
    light_store: &dyn LightStore,
    target_height: Height,
) -> bool {
    dbg!(
        target_height,
        light_store
            .all_verified()
            .into_iter()
            .map(|lb| lb.height())
            .collect::<Vec<_>>()
    );
    light_store.get_verified(target_height).is_some()
}

pub fn is_within_trust_period(
    light_block: &LightBlock,
    trusting_period: Duration,
    now: Time,
) -> bool {
    let header_time = light_block.signed_header.header.time;
    let expires_at = header_time + trusting_period;

    header_time < now && expires_at > now && header_time <= now
}
