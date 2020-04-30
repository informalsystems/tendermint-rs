use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::prelude::*;

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerifierError {
    #[error("invalid light block")]
    InvalidLightBlock(#[from] VerificationError),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerifierInput {
    VerifyLightBlock {
        trusted_state: TrustedState,
        light_block: LightBlock,
        options: VerificationOptions,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerifierOutput {
    ValidLightBlock(LightBlock),
}

pub type VerifierResult = Result<VerifierOutput, VerifierError>;

pub struct Verifier {
    predicates: Box<dyn VerificationPredicates>,
    voting_power_calculator: Box<dyn VotingPowerCalculator>,
    commit_validator: Box<dyn CommitValidator>,
    header_hasher: Box<dyn HeaderHasher>,
}

impl Verifier {
    pub fn new(
        predicates: impl VerificationPredicates + 'static,
        voting_power_calculator: impl VotingPowerCalculator + 'static,
        commit_validator: impl CommitValidator + 'static,
        header_hasher: impl HeaderHasher + 'static,
    ) -> Self {
        Self {
            predicates: Box::new(predicates),
            voting_power_calculator: Box::new(voting_power_calculator),
            commit_validator: Box::new(commit_validator),
            header_hasher: Box::new(header_hasher),
        }
    }

    pub fn process(&self, input: VerifierInput) -> VerifierResult {
        match input {
            VerifierInput::VerifyLightBlock {
                trusted_state,
                light_block,
                options,
            } => self.verify_light_block(light_block, trusted_state, options),
        }
    }

    pub fn verify_light_block(
        &self,
        light_block: LightBlock,
        trusted_state: TrustedState,
        options: VerificationOptions,
    ) -> Result<VerifierOutput, VerifierError> {
        self.predicates.verify_light_block(
            &self.voting_power_calculator,
            &self.commit_validator,
            &self.header_hasher,
            &trusted_state,
            &light_block,
            options,
        )?;

        // FIXME: Do we actually need to distinguish between LightBlock and TrustedState?
        let new_trusted_state = light_block.into();
        Ok(VerifierOutput::ValidLightBlock(new_trusted_state))
    }
}
