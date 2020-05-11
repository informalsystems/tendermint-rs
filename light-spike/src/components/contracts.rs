pub mod verify {
    use crate::prelude::*;

    pub fn trusted_state_contains_block_within_trusting_period(
        trusted_store: &StoreReader<Trusted>,
        trusting_period: Duration,
        now: Time,
    ) -> bool {
        trusted_store
            .all()
            .iter()
            .any(|lb| is_within_trust_period(lb, trusting_period, now))
    }

    pub fn target_height_greater_than_all_blocks_in_trusted_store(
        target_height: Height,
        trusted_store: &StoreReader<Trusted>,
    ) -> bool {
        trusted_store
            .all()
            .iter()
            .all(|lb| lb.height < target_height)
    }

    pub fn trusted_store_contains_block_at_target_height(
        target_height: Height,
        trusted_store: &StoreReader<Trusted>,
    ) -> bool {
        trusted_store.get(target_height).is_some()
    }

    fn is_within_trust_period(
        light_block: &LightBlock,
        trusting_period: Duration,
        now: Time,
    ) -> bool {
        let header_time = light_block.signed_header.header.time;
        let expires_at = header_time + trusting_period;

        header_time < now && expires_at > now && header_time <= now
    }
}

pub mod schedule {
    use crate::prelude::*;

    pub fn postcondition(
        trusted_state: &LightBlock,
        target_height: Height,
        next_height: Height,
        trusted_store: &StoreReader<Trusted>,
        untrusted_store: &StoreReader<Untrusted>,
    ) -> bool {
        let current_height = trusted_state.height;

        (next_height <= target_height)
            && ((next_height > current_height)
                || (next_height == current_height && current_height == target_height))
            && ((trusted_store.get(current_height).as_ref() == Some(trusted_state))
                || (untrusted_store.get(current_height).as_ref() == Some(trusted_state)))
    }
}
