use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::verifier::VerifierError;
use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum RpcRequest {
    FetchLightBlock(Height),
}

#[derive(Clone, Debug)]
pub enum RpcResponse {
    FetchedLightBlock(LightBlock),
}

pub type RpcChan = BiChan<RpcRequest, RpcResponse>;
pub type RpcEnd = BiChanEnd<RpcRequest, RpcResponse>;

#[derive(Clone, Debug)]
pub enum VerifierRequest {
    VerifyLightBlock {
        trusted_state: TrustedState,
        light_block: LightBlock,
        trust_threshold: TrustThreshold,
        trusting_period: Duration,
        now: SystemTime,
    },
}

#[derive(Clone, Debug)]
pub enum VerifierResponse {
    VerificationSucceeded(TrustedState),
    VerificationFailed(VerifierError),
}

pub type VerifierChan = BiChan<VerifierRequest, VerifierResponse>;
pub type VerifierEnd = BiChanEnd<VerifierRequest, VerifierResponse>;

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum DemuxerError {}
impl_event!(DemuxerError);

pub struct Demuxer {
    trusted_store: TSReadWriter,
    rpc: Rpc,
    rpc_end: RpcEnd,
    scheduler: Scheduler,
    verifier: Verifier,
    verifier_end: VerifierEnd,
}

impl Demuxer {
    pub fn new(
        trusted_store: TSReadWriter,
        rpc: Rpc,
        rpc_end: RpcEnd,
        scheduler: Scheduler,
        verifier: Verifier,
        verifier_end: VerifierEnd,
    ) -> Self {
        Self {
            trusted_store,
            rpc,
            rpc_end,
            scheduler,
            verifier,
            verifier_end,
        }
    }

    pub async fn verify_light_block(
        &mut self,
        trusted_state: TrustedState,
        light_block: LightBlock,
        trust_threshold: TrustThreshold,
        trusting_period: Duration,
        now: SystemTime,
    ) -> Result<Vec<TrustedState>, DemuxerError> {
        let result = self
            .scheduler
            .verify_light_block(
                trusted_state,
                light_block,
                trust_threshold,
                trusting_period,
                now,
            )
            .await;

        match result {
            Ok(new_trusted_states) => {
                for ts in &new_trusted_states {
                    self.trusted_store.add(ts.clone());
                }

                Ok(new_trusted_states)
            }
            Err(_err) => todo!(),
        }
    }

    pub async fn process_requests(&mut self) {
        loop {
            tokio::select! {
                request = self.verifier_end.next_request() => {
                    self.process_verifier_request(request)
                }
                request = self.rpc_end.next_request() => {
                    self.process_rpc_request(request)
                }
            }
        }
    }

    pub fn process_verifier_request(&mut self, request: VerifierRequest) {
        match request {
            VerifierRequest::VerifyLightBlock {
                trusted_state,
                light_block,
                trust_threshold,
                trusting_period,
                now,
            } => {
                let result = self.verifier.verify_light_block(
                    trusted_state,
                    light_block,
                    trust_threshold,
                    trusting_period,
                    now,
                );

                match result {
                    Ok(trusted_state) => self
                        .verifier_end
                        .reply(VerifierResponse::VerificationSucceeded(trusted_state)),
                    Err(err) => self
                        .verifier_end
                        .reply(VerifierResponse::VerificationFailed(err)),
                }
            }
        }
    }

    pub fn process_rpc_request(&mut self, request: RpcRequest) {
        match request {
            RpcRequest::FetchLightBlock(height) => {
                let result = self.rpc.fetch_light_block(height);
                match result {
                    Ok(light_block) => self
                        .rpc_end
                        .reply(RpcResponse::FetchedLightBlock(light_block)),
                    Err(_err) => todo!(),
                }
            }
        }
    }
}
