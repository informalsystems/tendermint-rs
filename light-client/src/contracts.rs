//! Predicates used in components contracts.

use crate::{
    store::LightStore,
    types::{Height, LightBlock, Status, Time},
};

use std::time::Duration;

pub fn trusted_store_contains_block_at_target_height<LB: LightBlock>(
    light_store: &dyn LightStore<LB>,
    target_height: Height,
) -> bool {
    light_store.get(target_height, Status::Verified).is_some()
        || light_store.get(target_height, Status::Trusted).is_some()
}

pub fn is_within_trust_period<LB: LightBlock>(
    light_block: &LB,
    trusting_period: Duration,
    now: Time,
) -> bool {
    let header_time = light_block.header_time();
    header_time > now - trusting_period
}

pub fn light_store_contains_block_within_trusting_period<LB: LightBlock>(
    light_store: &dyn LightStore<LB>,
    trusting_period: Duration,
    now: Time,
) -> bool {
    light_store
        .all(Status::Trusted)
        .any(|lb| is_within_trust_period(&lb, trusting_period, now))
}
