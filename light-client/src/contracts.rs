use crate::prelude::*;

pub fn trusted_store_contains_block_at_target_height(
    light_store: &dyn LightStore,
    target_height: Height,
) -> bool {
    light_store
        .get(target_height, VerifiedStatus::Verified)
        .is_some()
}

pub fn is_within_trust_period(
    light_block: &LightBlock,
    trusting_period: Duration,
    now: Time,
) -> bool {
    let header_time = light_block.signed_header.header.time;
    header_time > now - trusting_period
}

// pub fn trusted_state_contains_block_within_trusting_period(
//     light_store: &dyn LightStore,
//     trusting_period: Duration,
//     now: Time,
// ) -> bool {
//     light_store
//         .all(VerifiedStatus::Verified)
//         .any(|lb| is_within_trust_period(&lb, trusting_period, now))
// }

// pub fn target_height_greater_than_all_blocks_in_trusted_store(
//     light_store: &dyn LightStore,
//     target_height: Height,
// ) -> bool {
//     light_store
//         .all(VerifiedStatus::Verified)
//         .all(|lb| lb.height() < target_height)
// }

