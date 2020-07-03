//! Predicates used in components contracts.

use crate::{
    store::LightStore,
    types::{Height, LightBlock, Status, Time},
};

use std::time::Duration;

pub fn trusted_store_contains_block_at_target_height(
    light_store: &dyn LightStore,
    target_height: Height,
) -> bool {
    light_store.get(target_height, Status::Verified).is_some()
}

pub fn is_within_trust_period(
    light_block: &LightBlock,
    trusting_period: Duration,
    now: Time,
) -> bool {
    let header_time = light_block.signed_header.header.time;
    header_time > now - trusting_period
}

pub fn light_store_contains_block_within_trusting_period(
    light_store: &dyn LightStore,
    trusting_period: Duration,
    now: Time,
) -> bool {
    light_store
        .all(Status::Verified)
        .any(|lb| is_within_trust_period(&lb, trusting_period, now))
}

// pub fn target_height_greater_than_all_blocks_in_trusted_store(
//     light_store: &dyn LightStore,
//     target_height: Height,
// ) -> bool {
//     light_store
//         .all(Status::Verified)
//         .all(|lb| lb.height() < target_height)
// }
