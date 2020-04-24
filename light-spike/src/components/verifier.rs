use std::sync::mpsc::Sender;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::prelude::*;

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerifierError {
    #[error("invalid light block")]
    InvalidLightBlock(crate::errors::ErrorKind),
}

impl_event!(VerifierError);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerifierInput {
    VerifyUntrustedLightBlock(LightBlock),
}

impl_event!(VerifierInput);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerifierOutput {
    ValidLightBlock(TrustedState),
    NeedBisectionAt {
        pivot_height: Height,
        trusted_state: TrustedState,
        trust_threshold: TrustThreshold,
    },
}

impl_event!(VerifierOutput);

pub struct Verifier<VP> {
    trace: Sender<BoxedEvent>,
    predicates: VP,
    voting_power_calculator: Box<dyn VotingPowerCalculator>,
    commit_validator: Box<dyn CommitValidator>,
    header_hasher: Box<dyn HeaderHasher>,
}

impl<VP> Verifier<VP>
where
    VP: VerificationPredicates,
{
    pub fn new(
        trace: Sender<BoxedEvent>,
        predicates: VP,
        voting_power_calculator: impl VotingPowerCalculator + 'static,
        commit_validator: impl CommitValidator + 'static,
        header_hasher: impl HeaderHasher + 'static,
    ) -> Self {
        Self {
            trace,
            predicates,
            voting_power_calculator: Box::new(voting_power_calculator),
            commit_validator: Box::new(commit_validator),
            header_hasher: Box::new(header_hasher),
        }
    }

    pub fn verify_untrusted_light_block(
        &self,
        trusted_state: TrustedState,
        light_block: LightBlock,
        trust_threshold: TrustThreshold,
        trusting_period: Duration,
        now: SystemTime,
    ) -> Result<VerifierOutput, VerifierError> {
        self.trace(VerifierInput::VerifyUntrustedLightBlock(
            light_block.clone(),
        ));

        let verification_result = self.predicates.verify_untrusted_light_block(
            &self.voting_power_calculator,
            &self.commit_validator,
            &self.header_hasher,
            &trusted_state,
            &light_block,
            &trust_threshold,
            trusting_period,
            now,
        );

        match verification_result {
            Ok(()) => self.emit_verified_light_block(light_block),
            Err(e) => match e.kind() {
                ErrorKind::InsufficientVotingPower { .. } => {
                    self.on_insufficient_voting_power(trusted_state, light_block, trust_threshold)
                }
                kind => {
                    let output = VerifierError::InvalidLightBlock(kind.to_owned());
                    self.trace(output.clone());
                    Err(output)
                }
            },
        }
    }

    fn emit_verified_light_block(
        &self,
        light_block: LightBlock,
    ) -> Result<VerifierOutput, VerifierError> {
        let new_trusted_state = TrustedState {
            header: light_block.signed_header.header,
            validators: light_block.validator_set,
        };

        let output = VerifierOutput::ValidLightBlock(new_trusted_state);

        self.trace(output.clone());

        Ok(output)
    }

    pub fn on_insufficient_voting_power(
        &self,
        trusted_state: TrustedState,
        light_block: LightBlock,
        trust_threshold: TrustThreshold,
    ) -> Result<VerifierOutput, VerifierError> {
        // Get the pivot height for bisection.
        let trusted_height = trusted_state.header.height;
        let untrusted_height = light_block.height;
        let pivot_height = trusted_height
            .checked_add(untrusted_height)
            .expect("height overflow")
            / 2;

        let output = VerifierOutput::NeedBisectionAt {
            pivot_height,
            trusted_state,
            trust_threshold,
        };

        self.trace(output.clone());

        Ok(output)
    }

    fn trace(&self, e: impl Event + 'static) {
        self.trace.send(Box::new(e)).expect("could not trace event");
    }
}
