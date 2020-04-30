use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VerifierInput {
    VerifyLightBlock {
        trusted_state: TrustedState,
        light_block: LightBlock,
        options: VerificationOptions,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VerifierOutput {
    Success,
    NotEnoughTrust,
    Invalid(VerificationError),
}

pub trait Verifier {
    fn process(&self, input: VerifierInput) -> VerifierOutput;
}

impl<F> Verifier for F
where
    F: Fn(VerifierInput) -> VerifierOutput,
{
    fn process(&self, input: VerifierInput) -> VerifierOutput {
        self(input)
    }
}

pub struct RealVerifier {
    predicates: Box<dyn VerificationPredicates>,
    voting_power_calculator: Box<dyn VotingPowerCalculator>,
    commit_validator: Box<dyn CommitValidator>,
    header_hasher: Box<dyn HeaderHasher>,
}

impl Verifier for RealVerifier {
    fn process(&self, input: VerifierInput) -> VerifierOutput {
        match input {
            VerifierInput::VerifyLightBlock {
                trusted_state,
                light_block,
                options,
            } => self.verify_light_block(light_block, trusted_state, options),
        }
    }
}

impl RealVerifier {
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

    pub fn verify_light_block(
        &self,
        light_block: LightBlock,
        trusted_state: TrustedState,
        options: VerificationOptions,
    ) -> VerifierOutput {
        let result = crate::predicates::verify_light_block(
            &*self.predicates,
            &self.voting_power_calculator,
            &self.commit_validator,
            &self.header_hasher,
            &trusted_state,
            &light_block,
            options,
        );

        match result {
            Ok(()) => VerifierOutput::Success,
            Err(VerificationError::InsufficientVotingPower { .. }) => {
                VerifierOutput::NotEnoughTrust
            }
            Err(e) => VerifierOutput::Invalid(e),
        }
    }
}
