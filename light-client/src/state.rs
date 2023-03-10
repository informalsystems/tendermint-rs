//! State maintained by the light client.

use std::collections::{HashMap, HashSet};

use contracts::*;

use crate::{
    store::LightStore,
    verifier::types::{Height, LightBlock},
};

/// Records which blocks were needed to verify a target block, eg. during bisection.
pub type VerificationTrace = HashMap<Height, HashSet<Height>>;

/// The state managed by the light client.
#[derive(Debug)]
pub struct State {
    /// Store for light blocks.
    pub light_store: Box<dyn LightStore>,

    /// Records which blocks were needed to verify a target block, eg. during bisection.
    pub verification_trace: VerificationTrace,
}

impl State {
    /// Create a new state from the given light store with an empty verification trace.
    pub fn new(light_store: impl LightStore + 'static) -> Self {
        Self {
            light_store: Box::new(light_store),
            verification_trace: VerificationTrace::new(),
        }
    }

    /// Record that the block at `height` was needed to verify the block at `target_height`.
    ///
    /// ## Preconditions
    /// - `height` <= `target_height`
    #[requires(height <= target_height)]
    pub fn trace_block(&mut self, target_height: Height, height: Height) {
        self.verification_trace
            .entry(target_height)
            .or_insert_with(|| {
                let mut trace = HashSet::new();
                trace.insert(target_height);
                trace
            })
            .insert(height);
    }

    /// Get the verification trace for the block at `target_height`.
    pub fn get_trace(&self, target_height: Height) -> Vec<LightBlock> {
        use std::cmp::Reverse;

        let mut trace = self
            .verification_trace
            .get(&target_height)
            .unwrap_or(&HashSet::new())
            .iter()
            .flat_map(|&height| self.light_store.get_trusted_or_verified(height))
            .collect::<Vec<_>>();

        trace.sort_by_key(|lb| Reverse(lb.height()));
        trace
    }
}
