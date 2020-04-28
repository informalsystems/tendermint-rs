use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum RpcRequest {
    FetchLightBlock(Height),
}

#[derive(Clone, Debug)]
pub enum RpcResponse {
    FetchedLightBlock(LightBlock),
}

#[derive(Clone, Debug)]
pub enum VerifierRequest {
    VerifyLightBlock {
        trusted_state: TrustedState,
        light_block: LightBlock,
        options: VerificationOptions,
    },
}

#[derive(Clone, Debug)]
pub enum VerifierResponse {
    VerificationSucceeded(TrustedState),
    VerificationFailed(VerifierError),
}

pub trait Router {
    fn query_rpc(&self, request: RpcRequest) -> RpcResponse;
    fn query_verifier(&self, request: VerifierRequest) -> VerifierResponse;
}
