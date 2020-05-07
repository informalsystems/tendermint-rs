use crate::prelude::*;

pub trait Scheduler {
    fn schedule(&self, light_block: &LightBlock, trusted_state: &TrustedState) -> Height;
}

impl<F> Scheduler for F
where
    F: Fn(&LightBlock, &TrustedState) -> Height,
{
    fn schedule(&self, light_block: &LightBlock, trusted_state: &TrustedState) -> Height {
        self(light_block, trusted_state)
    }
}

pub fn schedule(light_block: &LightBlock, trusted_state: &TrustedState) -> Height {
    let trusted_height = trusted_state.height;
    let untrusted_height = light_block.height;

    assert!(trusted_height < untrusted_height);

    // Equivalent to (trusted_height + untrusted_height) / 2, but avoid overflows
    trusted_height + (untrusted_height - trusted_height) / 2
}
