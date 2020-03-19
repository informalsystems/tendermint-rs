use async_trait::async_trait;

use futures::stream::StreamExt;
use tendermint::block::signed_header::SignedHeader as TMCommit;
use tendermint::block::Header as TMHeader;
use tendermint::lite::{error, Height, SignedHeader};
use tendermint::rpc;
use tendermint::validator;
use tendermint::validator::Set;
use tendermint::{block, lite};
use tokio::sync::mpsc;
use tokio::sync::oneshot;

/// RPCRequester wraps the Tendermint rpc::Client.
#[derive(Clone)]
pub struct RPCRequester {
    rpc_request_sender: mpsc::Sender<RPCRequest>,
}

impl RPCRequester {
    pub fn new(client: rpc::Client) -> Self {
        let rpc_request_sender = RPCRequesterTask::new(client);
        RPCRequester { rpc_request_sender }
    }
}

enum RPCRequest {
    SignedHeader(Height, oneshot::Sender<RPCResponse>),
    ValidatorSet(Height, oneshot::Sender<RPCResponse>),
}

enum RPCResponse {
    SignedHeader(TMSignedHeader),
    ValidatorSet(Set),
}

struct RPCRequesterTask {
    client: rpc::Client,
}

impl RPCRequesterTask {
    pub fn new(client: rpc::Client) -> mpsc::Sender<RPCRequest> {
        let (sender, mut receiver) = mpsc::channel(1);
        let mut receiver = receiver.fuse();
        tokio::spawn(async move {
            loop {
                select! {
                    new_request = receiver.select_next_some() => {
                        match new_request {
                            RPCRequest::SignedHeader(height, sender) => {
                                let height: block::Height = height.into();
                                let r = match height.value() {
                                    0 => client.latest_commit().await,
                                    _ => client.commit(height).await,
                                };
                                let r = r.expect("Failed to do RPC.");
                                // Note: why do we hit a recursion limit with this?
                                let _ = sender.send(RPCResponse::SignedHeader(r.signed_header.into()));
                            },
                            RPCRequest::ValidatorSet(height, sender) => {
                                let r = client.validators(height).await.expect("Failed to do RPC.");
                                let _ = sender.send(RPCResponse::ValidatorSet(Set::new(r.validators)));
                            }
                        }
                    }
                    complete => break,
                }
            }
        });
        sender
    }
}

type TMSignedHeader = SignedHeader<TMCommit, TMHeader>;

#[async_trait]
impl lite::types::Requester<TMCommit, TMHeader> for RPCRequester {
    /// Request the signed header at height h.
    /// If h==0, request the latest signed header.
    /// TODO: use an enum instead of h==0.
    async fn signed_header(&mut self, h: Height) -> Result<TMSignedHeader, error::Error> {
        let (sender, mut receiver) = oneshot::channel();
        self.rpc_request_sender
            .send(RPCRequest::SignedHeader(h, sender))
            .await;
        match receiver.await {
            Ok(RPCResponse::SignedHeader(headers)) => Ok(headers.into()),
            Err(error) => Err(error::Kind::RequestFailed.context(error).into()),
            _ => panic!("Unexpected"),
        }
    }

    /// Request the validator set at height h.
    async fn validator_set(&mut self, h: Height) -> Result<Set, error::Error> {
        let (sender, mut receiver) = oneshot::channel();
        self.rpc_request_sender
            .send(RPCRequest::SignedHeader(h, sender))
            .await;
        match receiver.await {
            Ok(RPCResponse::ValidatorSet(set)) => {
                Ok(validator::Set::new(set.validators().to_vec()))
            }
            Err(error) => Err(error::Kind::RequestFailed.context(error).into()),
            _ => panic!("Unexpected"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tendermint::lite::types::Header as LiteHeader;
    use tendermint::lite::types::Requester as LiteRequester;
    use tendermint::lite::types::ValidatorSet as LiteValSet;
    use tendermint::rpc;

    // TODO: integration test
    #[tokio::test]
    #[ignore]
    async fn test_val_set() {
        let client = rpc::Client::new("localhost:26657".parse().unwrap());
        let req = RPCRequester::new(client);
        let r1 = req.validator_set(5).await.unwrap();
        let r2 = req.signed_header(5).await.unwrap();
        assert_eq!(r1.hash(), r2.header().validators_hash());
    }
}
