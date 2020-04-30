use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::verifier::VerifierError;
use crate::prelude::*;

#[async_trait::async_trait(?Send)]
pub trait Scheduler {
    async fn process(
        &self,
        trusted_store: TSReader,
        input: SchedulerInput,
        co: Co<SchedulerRequest, SchedulerResponse>,
    ) -> SchedulerResult;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SchedulerInput {
    VerifyHeight {
        height: Height,
        trusted_state: TrustedState,
        options: VerificationOptions,
    },
    VerifyLightBlock {
        light_block: LightBlock,
        trusted_state: TrustedState,
        options: VerificationOptions,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SchedulerOutput {
    TrustedStates(Vec<TrustedState>),
}

#[derive(Clone, Debug, Error, PartialEq, Serialize, Deserialize)]
pub enum SchedulerError {
    #[error("invalid light block {0} because: {1}")]
    InvalidLightBlock(LightBlock, VerifierError),
}

pub enum SchedulerRequest {
    GetLightBlock(Height),
    VerifyLightBlock {
        light_block: LightBlock,
        trusted_state: TrustedState,
        options: VerificationOptions,
    },
    ValidateLightBlock {
        light_block: LightBlock,
        trusted_state: TrustedState,
        options: VerificationOptions,
    },
}

pub enum SchedulerResponse {
    Init,
    LightBlock(LightBlock),
    Validated(Result<LightBlock, VerifierError>),
    Verified(Result<Vec<LightBlock>, VerifierError>),
}

pub type SchedulerResult = Result<SchedulerOutput, SchedulerError>;

pub struct RealScheduler;

#[async_trait::async_trait(?Send)]
impl Scheduler for RealScheduler {
    async fn process(
        &self,
        trusted_store: TSReader,
        input: SchedulerInput,
        co: Co<SchedulerRequest, SchedulerResponse>,
    ) -> SchedulerResult {
        match input {
            SchedulerInput::VerifyHeight {
                height,
                trusted_state,
                options,
            } => verify_height(height, trusted_state, options, trusted_store, co).await,

            SchedulerInput::VerifyLightBlock {
                light_block,
                trusted_state,
                options,
            } => verify_light_block(light_block, trusted_state, options, trusted_store, co).await,
        }
    }
}

pub async fn verify_height(
    height: Height,
    trusted_state: TrustedState,
    options: VerificationOptions,
    trusted_store: TSReader,
    co: Co<SchedulerRequest, SchedulerResponse>,
) -> SchedulerResult {
    if let Some(trusted_state) = trusted_store.get(height) {
        Ok(SchedulerOutput::TrustedStates(vec![trusted_state]))
    } else {
        let response = co.yield_(SchedulerRequest::GetLightBlock(height)).await;
        let light_block = unwrap!(SchedulerResponse::LightBlock, response);

        verify_light_block(light_block, trusted_state, options, trusted_store, co).await
    }
}

pub async fn verify_light_block(
    light_block: LightBlock,
    trusted_state: TrustedState,
    options: VerificationOptions,
    trusted_store: TSReader,
    co: Co<SchedulerRequest, SchedulerResponse>,
) -> SchedulerResult {
    if let Some(in_store) = trusted_store.get(light_block.height) {
        return Ok(SchedulerOutput::TrustedStates(vec![in_store]));
    }

    let response = co
        .yield_(SchedulerRequest::ValidateLightBlock {
            light_block: light_block.clone(),
            trusted_state: trusted_state.clone(),
            options: options.clone(),
        })
        .await;

    let result = unwrap!(SchedulerResponse::Validated, response);
    match result {
        Err(err) if not_enough_trust(&err) => {
            do_bisection(light_block, trusted_state, options, co).await
        }
        Err(err) => Err(SchedulerError::InvalidLightBlock(light_block, err)),
        Ok(light_block) => {
            let trusted_state = light_block.into();
            Ok(SchedulerOutput::TrustedStates(vec![trusted_state]))
        }
    }
}

fn not_enough_trust(e: &VerifierError) -> bool {
    match e {
        VerifierError::InvalidLightBlock(e) => e.not_enough_trust(),
    }
}

fn compute_pivot_height(light_block: &LightBlock, trusted_state: &TrustedState) -> Height {
    let trusted_height = trusted_state.height;
    let untrusted_height = light_block.height;
    let pivot_height = trusted_height
        .checked_add(untrusted_height)
        .expect("height overflow")
        / 2;

    pivot_height
}

pub async fn do_bisection(
    light_block: LightBlock,
    trusted_state: TrustedState,
    options: VerificationOptions,
    co: Co<SchedulerRequest, SchedulerResponse>,
) -> SchedulerResult {
    let pivot_height = compute_pivot_height(&light_block, &trusted_state);

    let pivot_lb = co
        .yield_(SchedulerRequest::GetLightBlock(pivot_height))
        .await;

    let pivot_light_block = unwrap!(SchedulerResponse::LightBlock, pivot_lb);

    let pivot_response = co
        .yield_(SchedulerRequest::VerifyLightBlock {
            light_block: pivot_light_block.clone(),
            trusted_state: trusted_state.clone(),
            options: options.clone(),
        })
        .await;

    let mut valid_light_blocks = unwrap!(SchedulerResponse::Verified, pivot_response)
        .map_err(|e| SchedulerError::InvalidLightBlock(pivot_light_block, e))?;

    let lb_response = co
        .yield_(SchedulerRequest::ValidateLightBlock {
            light_block: light_block.clone(),
            trusted_state,
            options,
        })
        .await;

    let valid_light_block = unwrap!(SchedulerResponse::Validated, lb_response)
        .map_err(|e| SchedulerError::InvalidLightBlock(light_block, e))?;

    valid_light_blocks.push(valid_light_block);

    let trusted_states = valid_light_blocks.into_iter().map(Into::into).collect();
    Ok(SchedulerOutput::TrustedStates(trusted_states))
}
