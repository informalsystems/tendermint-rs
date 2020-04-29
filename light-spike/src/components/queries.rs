use crate::prelude::*;

pub fn query_rpc(rpc: &Rpc, request: RpcRequest) -> RpcResponse {
    match request {
        RpcRequest::FetchLightBlock(height) => {
            let result = rpc.fetch_light_block(height);
            match result {
                Ok(RpcOutput::FetchedLightBlock(light_block)) => {
                    RpcResponse::FetchedLightBlock(light_block)
                }
                Err(_err) => todo!(),
            }
        }
    }
}

pub fn query_verifier(verifier: &Verifier, request: VerifierRequest) -> VerifierResponse {
    match request {
        VerifierRequest::VerifyLightBlock {
            trusted_state,
            light_block,
            options,
        } => {
            let result = verifier.verify_light_block(trusted_state, light_block, options);

            match result {
                Ok(VerifierOutput::ValidLightBlock(trusted_state)) => {
                    VerifierResponse::VerificationSucceeded(trusted_state)
                }
                Err(err) => VerifierResponse::VerificationFailed(err),
            }
        }
    }
}
