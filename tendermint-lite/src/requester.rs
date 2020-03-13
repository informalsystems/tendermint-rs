use async_trait::async_trait;

use tendermint::block::signed_header::SignedHeader as TMCommit;
use tendermint::block::Header as TMHeader;
use tendermint::lite::{error, Height, SignedHeader};
use tendermint::rpc;
use tendermint::validator;
use tendermint::validator::Set;
use tendermint::{block, lite};

/// RPCRequester wraps the Tendermint rpc::Client.
pub struct RPCRequester {
    client: rpc::Client,
}

impl RPCRequester {
    pub fn new(client: rpc::Client) -> Self {
        RPCRequester { client }
    }
}

type TMSignedHeader = SignedHeader<TMCommit, TMHeader>;

#[async_trait]
impl lite::types::Requester<TMCommit, TMHeader> for RPCRequester {
    /// Request the signed header at height h.
    /// If h==0, request the latest signed header.
    /// TODO: use an enum instead of h==0.
    async fn signed_header(&self, h: Height) -> Result<TMSignedHeader, error::Error> {
        let height: block::Height = h.into();
        let r = match height.value() {
            0 => self.client.latest_commit().await,
            _ => self.client.commit(height).await,
        };
        match r {
            Ok(response) => Ok(response.signed_header.into()),
            Err(error) => Err(error::Kind::RequestFailed.context(error).into()),
        }
    }

    /// Request the validator set at height h.
    async fn validator_set(&self, h: Height) -> Result<Set, error::Error> {
        let r = self.client.validators(h).await;
        match r {
            Ok(response) => Ok(validator::Set::new(response.validators)),
            Err(error) => Err(error::Kind::RequestFailed.context(error).into()),
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
