// FIXME: Figure out a way to decouple components

use std::sync::mpsc::Sender;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::rpc::RpcError;
use super::verifier::VerifierError;
use crate::prelude::*;

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulerError {
    #[error("RPC error")]
    RpcError(RpcError),
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

pub struct Scheduler<VP> {
    trace: Sender<BoxedEvent>,
    rpc: Rpc,
    verifier: Verifier<VP>,
    trusted_store: TSReader,
}

impl<VP> Scheduler<VP>
where
    VP: VerificationPredicates,
{
    pub fn new(
        trace: Sender<BoxedEvent>,
        rpc: Rpc,
        verifier: Verifier<VP>,
        trusted_store: TSReader,
    ) -> Self {
        Self {
            trace,
            rpc,
            verifier,
            trusted_store,
        }
    }

    pub fn verify_untrusted_light_block(
        &mut self,
        trusted_state: TrustedState,
        light_block: LightBlock,
        trust_threshold: TrustThreshold,
        trusting_period: Duration,
        now: SystemTime,
    ) -> Result<Vec<TrustedState>, SchedulerError> {
        self.trace(SchedulerInput::VerifyUntrustedLightBlock(
            light_block.clone(),
        ));

        if let Some(trusted_state_in_store) = self.trusted_store.get(light_block.height) {
            let output = vec![trusted_state_in_store];
            self.trace(SchedulerOutput::ValidLightBlock(output.clone()));
            return Ok(output);
        }

        let verifier_result = self.verifier.verify_untrusted_light_block(
            trusted_state.clone(),
            light_block.clone(),
            trust_threshold,
            trusting_period,
            now,
        );

        match verifier_result {
            Ok(trusted_state) => self.verification_succeded(trusted_state),
            Err(VerifierError::InvalidLightBlock(ErrorKind::InsufficientVotingPower {
                ..
            })) => self.perform_bisection(
                trusted_state,
                light_block,
                trust_threshold,
                trusting_period,
                now,
            ),
            Err(err) => {
                let output = SchedulerError::InvalidLightBlock(err);
                self.trace(output.clone());
                Err(output)
            }
        }
    }

    fn verification_succeded(
        &mut self,
        new_trusted_state: TrustedState,
    ) -> Result<Vec<TrustedState>, SchedulerError> {
        self.trace(SchedulerOutput::ValidLightBlock(vec![
            new_trusted_state.clone()
        ]));

        Ok(vec![new_trusted_state])
    }

    pub fn perform_bisection(
        &mut self,
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

        self.trace(SchedulerOutput::PerformBisectionAt {
            pivot_height,
            trust_threshold,
            trusted_state: trusted_state.clone(),
        });

        let pivot_light_block = self
            .rpc
            .fetch_light_block(pivot_height)
            .map_err(SchedulerError::RpcError)?;

        let mut pivot_trusted_states = self.verify_untrusted_light_block(
            trusted_state,
            pivot_light_block,
            trust_threshold,
            trusting_period,
            now,
        )?;

        let trusted_state_left = pivot_trusted_states.last().cloned().unwrap(); // FIXME: Unwrap

        let mut new_trusted_states = self.verify_untrusted_light_block(
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

    fn trace(&self, e: impl Event + 'static) {
        self.trace.send(Box::new(e)).expect("could not trace event");
    }
}
