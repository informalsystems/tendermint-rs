use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::prelude::*;

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum DemuxerError {}
impl_event!(DemuxerError);

pub struct Demuxer {
    pub(super) trusted_store: TSReadWriter,
    pub(super) rpc: Rpc,
    pub(super) scheduler: Scheduler,
    pub(super) verifier: Verifier,
}

impl Router for &mut Demuxer {
    fn query_rpc(&self, request: RpcRequest) -> RpcResponse {
        match request {
            RpcRequest::FetchLightBlock(height) => {
                let result = self.rpc.fetch_light_block(height);
                match result {
                    Ok(RpcOutput::FetchedLightBlock(light_block)) => {
                        RpcResponse::FetchedLightBlock(light_block)
                    }
                    Err(_err) => todo!(),
                }
            }
        }
    }

    fn query_verifier(&self, request: VerifierRequest) -> VerifierResponse {
        match request {
            VerifierRequest::VerifyLightBlock {
                trusted_state,
                light_block,
                options,
            } => {
                let result = self
                    .verifier
                    .verify_light_block(trusted_state, light_block, options);

                match result {
                    Ok(VerifierOutput::ValidLightBlock(trusted_state)) => {
                        VerifierResponse::VerificationSucceeded(trusted_state)
                    }
                    Err(err) => VerifierResponse::VerificationFailed(err),
                }
            }
        }
    }
}

impl Demuxer {
    pub fn new(
        trusted_store: TSReadWriter,
        rpc: Rpc,
        scheduler: Scheduler,
        verifier: Verifier,
    ) -> Self {
        Self {
            trusted_store,
            rpc,
            scheduler,
            verifier,
        }
    }

    pub fn verify_light_block(
        &mut self,
        trusted_state: TrustedState,
        light_block: LightBlock,
        options: VerificationOptions,
    ) -> Result<Vec<TrustedState>, DemuxerError> {
        let result = self
            .scheduler
            .verify_light_block(&self, trusted_state, light_block, options);

        match result {
            Ok(SchedulerOutput::ValidLightBlock(new_trusted_states)) => {
                for ts in &new_trusted_states {
                    self.trusted_store.add(ts.clone());
                }

                Ok(new_trusted_states)
            }
            Err(_err) => todo!(),
        }
    }
}
