use serde::{Deserialize, Serialize};

use crate::prelude::*;

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
        primary: &LightClient,
        secondaries: Vec<&LightClient>,
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
        primary: &LightClient,
        secondaries: Vec<&LightClient>,
    ) -> ForkDetection {
        let mut forks = Vec::with_capacity(secondaries.len());

        for secondary in secondaries {
            let mut state: State = todo();
            let secondary_block = secondary
                .get_or_fetch_block(light_block.height(), &mut state)
                .unwrap(); // FIXME: unwrap

            // TODO: Should hash the headers here instead of comparing them for equality?
            if light_block.signed_header.header == secondary_block.signed_header.header {
                continue;
            }

            let latest_trusted = primary
                .state
                .light_store
                .latest(VerifiedStatus::Verified)
                .unwrap(); // FIXME: unwrap

            state
                .light_store
                .update(latest_trusted, VerifiedStatus::Verified);

            state
                .light_store
                .update(secondary_block.clone(), VerifiedStatus::Unverified);

            let result = secondary.verify_to_target_with_state(light_block.height(), &mut state);

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
