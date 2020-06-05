use serde::{Deserialize, Serialize};

use crate::prelude::*;
use crate::supervisor::Instance;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ForkDetection {
    Detected(Vec<Fork>),
    NotDetected,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Fork {
    Forked(LightBlock),
    Faulty(LightBlock),
}

pub trait ForkDetector {
    fn detect_forks(
        &self,
        light_block: &LightBlock,
        trusted_state: &LightBlock,
        secondaries: Vec<&Instance>,
    ) -> ForkDetection;
}

pub struct ProdForkDetector {}

impl ProdForkDetector {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ProdForkDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ForkDetector for ProdForkDetector {
    fn detect_forks(
        &self,
        light_block: &LightBlock,
        trusted_state: &LightBlock,
        secondaries: Vec<&Instance>,
    ) -> ForkDetection {
        let mut forks = Vec::with_capacity(secondaries.len());

        for secondary in secondaries {
            let mut state = State::new(MemoryStore::new());

            let secondary_block = secondary
                .light_client
                .get_or_fetch_block(light_block.height(), &mut state)
                .unwrap(); // FIXME: unwrap

            // TODO: Should hash the headers here instead of comparing them for equality?
            if light_block.signed_header.header == secondary_block.signed_header.header {
                continue;
            }

            state
                .light_store
                .update(trusted_state.clone(), VerifiedStatus::Verified);

            state
                .light_store
                .update(secondary_block.clone(), VerifiedStatus::Unverified);

            let result = secondary
                .light_client
                .verify_to_target(light_block.height(), &mut state);

            match result {
                Ok(_) => forks.push(Fork::Forked(secondary_block)),
                Err(e) if e.kind().has_expired() => forks.push(Fork::Forked(secondary_block)),
                Err(_) => forks.push(Fork::Faulty(secondary_block)),
            }
        }

        if forks.is_empty() {
            ForkDetection::NotDetected
        } else {
            ForkDetection::Detected(forks)
        }
    }
}
