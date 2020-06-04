use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ForkDetection {
    NoFork,
}

pub trait ForkDetector {
    fn detect_forks(
        &self,
        light_block: &LightBlock,
        primary: LightClient,
        secondaries: Vec<LightClient>,
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
        primary: LightClient,
        secondaries: Vec<LightClient>,
    ) -> ForkDetection {
        for mut secondary in secondaries {
            let secondary_block = secondary.get_or_fetch_block(light_block.height()).unwrap(); // FIXME: unwrap

            if light_block.signed_header.header == secondary_block.signed_header.header {
                // Header matches, we continue.
                continue;
            }

            let latest_trusted = primary
                .state
                .light_store
                .latest(VerifiedStatus::Verified)
                .unwrap(); // FIXME: unwrap

            secondary
                .state
                .light_store
                .update(latest_trusted, VerifiedStatus::Verified);

            secondary
                .state
                .light_store
                .update(secondary_block, VerifiedStatus::Unverified);

            let result = secondary.verify_to_target(light_block.height());

            // TODO: Handle case where block expired
            match result {
                Ok(_) => todo!(),  // There is a fork, report `secondary_block`
                Err(_) => todo!(), // `secondary` is faulty, report it
            }
        }

        ForkDetection::NoFork
    }
}
