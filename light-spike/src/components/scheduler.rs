use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::prelude::*;

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulerError {
    #[error("invalid light block: {0}")]
    InvalidLightBlock(VerifierError),
}

impl_event!(SchedulerError);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulerInput {
    VerifyUntrustedLightBlock(LightBlock),
}

impl_event!(SchedulerInput);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulerOutput {
    ValidLightBlock(Vec<TrustedState>),
    PerformBisectionAt {
        pivot_height: Height,
        trusted_state: TrustedState,
        trust_threshold: TrustThreshold,
    },
}

impl_event!(SchedulerOutput);

pub struct Scheduler {
    trusted_store: TSReader,
}

impl Scheduler {
    pub fn new(trusted_store: TSReader) -> Self {
        Self { trusted_store }
    }

    pub fn verify_light_block(
        &self,
        router: &impl Router,
        trusted_state: TrustedState,
        light_block: LightBlock,
        trust_threshold: TrustThreshold,
        trusting_period: Duration,
        now: SystemTime,
    ) -> Result<Vec<TrustedState>, SchedulerError> {
        if let Some(trusted_state_in_store) = self.trusted_store.get(light_block.height) {
            let output = vec![trusted_state_in_store];
            return Ok(output);
        }

        let verifier_result = self.perform_verify_light_block(
            router,
            trusted_state.clone(),
            light_block.clone(),
            trust_threshold,
            trusting_period,
            now,
        );

        match verifier_result {
            VerifierResponse::VerificationSucceeded(trusted_state) => {
                self.verification_succeded(trusted_state)
            }
            VerifierResponse::VerificationFailed(err) => self.verification_failed(
                router,
                err,
                trusted_state,
                light_block,
                trust_threshold,
                trusting_period,
                now,
            ),
        }
    }

    fn perform_verify_light_block(
        &self,
        router: &impl Router,
        trusted_state: TrustedState,
        light_block: LightBlock,
        trust_threshold: TrustThreshold,
        trusting_period: Duration,
        now: SystemTime,
    ) -> VerifierResponse {
        router.query_verifier(VerifierRequest::VerifyLightBlock {
            trusted_state,
            light_block,
            trust_threshold,
            trusting_period,
            now,
        })
    }

    fn verification_succeded(
        &self,
        new_trusted_state: TrustedState,
    ) -> Result<Vec<TrustedState>, SchedulerError> {
        Ok(vec![new_trusted_state])
    }

    fn verification_failed(
        &self,
        router: &impl Router,
        err: VerifierError,
        trusted_state: TrustedState,
        light_block: LightBlock,
        trust_threshold: TrustThreshold,
        trusting_period: Duration,
        now: SystemTime,
    ) -> Result<Vec<TrustedState>, SchedulerError> {
        match err {
            VerifierError::InvalidLightBlock(ErrorKind::InsufficientVotingPower { .. }) => self
                .perform_bisection(
                    router,
                    trusted_state,
                    light_block,
                    trust_threshold,
                    trusting_period,
                    now,
                ),
            err => {
                let output = SchedulerError::InvalidLightBlock(err);
                Err(output)
            }
        }
    }

    fn perform_bisection(
        &self,
        router: &impl Router,
        trusted_state: TrustedState,
        light_block: LightBlock,
        trust_threshold: TrustThreshold,
        trusting_period: Duration,
        now: SystemTime,
    ) -> Result<Vec<TrustedState>, SchedulerError> {
        // Get the pivot height for bisection.
        let trusted_height = trusted_state.header.height;
        let untrusted_height = light_block.height;
        let pivot_height = trusted_height
            .checked_add(untrusted_height)
            .expect("height overflow")
            / 2;

        let pivot_light_block = self.request_fetch_light_block(router, pivot_height)?;

        let mut pivot_trusted_states = self.verify_light_block(
            router,
            trusted_state,
            pivot_light_block,
            trust_threshold,
            trusting_period,
            now,
        )?;

        let trusted_state_left = pivot_trusted_states.last().cloned().unwrap(); // FIXME: Unwrap

        let mut new_trusted_states = self.verify_light_block(
            router,
            trusted_state_left,
            light_block,
            trust_threshold,
            trusting_period,
            now,
        )?;

        new_trusted_states.append(&mut pivot_trusted_states);
        new_trusted_states.sort_by_key(|ts| ts.header.height);

        Ok(new_trusted_states)
    }

    fn request_fetch_light_block(
        &self,
        router: &impl Router,
        height: Height,
    ) -> Result<LightBlock, SchedulerError> {
        let rpc_response = router.query_rpc(RpcRequest::FetchLightBlock(height));

        match rpc_response {
            RpcResponse::FetchedLightBlock(light_block) => Ok(light_block),
        }
    }
}
