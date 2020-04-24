// FIXME: Figure out how to get rid of type parameter

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
    ) -> Result<TrustedState, VerifierError> {
        self.trace(VerifierInput::VerifyUntrustedLightBlock(
            light_block.clone(),
        ));

        self.predicates
            .verify_untrusted_light_block(
                &self.voting_power_calculator,
                &self.commit_validator,
                &self.header_hasher,
                &trusted_state,
                &light_block,
                &trust_threshold,
                trusting_period,
                now,
            )
            .map_err(|e| VerifierError::InvalidLightBlock(e.kind().to_owned()))?;

        let new_trusted_state = TrustedState {
            header: light_block.signed_header.header,
            validators: light_block.validator_set,
        };

        self.trace(VerifierOutput::ValidLightBlock(new_trusted_state.clone()));

        Ok(new_trusted_state)
    }

    fn trace(&self, e: impl Event + 'static) {
        self.trace.send(Box::new(e)).expect("could not trace event");
    }
}
