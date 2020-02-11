use tendermint::block::signed_header::SignedHeader as TMCommit;
use tendermint::block::Header as TMHeader;
use tendermint::rpc;
use tendermint::validator;
use tendermint::{block, lite};

use core::future::Future;
use tendermint::lite::{error, Height, SignedHeader};
use tendermint::validator::Set;
use tokio::runtime::Builder;

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

impl lite::types::Requester<TMCommit, TMHeader> for RPCRequester {
    /// Request the signed header at height h.
    /// If h==0, request the latest signed header.
    /// TODO: use an enum instead of h==0.
    fn signed_header(&self, h: Height) -> Result<TMSignedHeader, error::ErrorKind> {
        let height: block::Height = h.into();
        let r = match height.value() {
            0 => block_on(self.client.latest_commit()),
            _ => block_on(self.client.commit(height)),
        };
        match r {
            Ok(response) => Ok(response.signed_header.into()),
            Err(error) => Err(error::ErrorKind::RequestFailed(format!("{:?}", error))),
        }
    }

    /// Request the validator set at height h.
    fn validator_set(&self, h: Height) -> Result<Set, error::ErrorKind> {
        let r = block_on(self.client.validators(h));
        match r {
            Ok(response) => Ok(validator::Set::new(response.validators)),
            Err(error) => Err(error::ErrorKind::RequestFailed(format!("{:?}", error))),
        }
    }
}

pub fn block_on<F: Future>(future: F) -> F::Output {
    Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tendermint::lite::types::Header as LiteHeader;
    use tendermint::lite::types::Requester as LiteRequester;
    use tendermint::lite::types::ValidatorSet as LiteValSet;
    use tendermint::rpc;

    // TODO: integration test
    #[test]
    #[ignore]
    fn test_val_set() {
        let client = block_on(rpc::Client::new(&"localhost:26657".parse().unwrap())).unwrap();
        let req = RPCRequester::new(client);
        let r1 = req.validator_set(5).unwrap();
        let r2 = req.signed_header(5).unwrap();
        assert_eq!(r1.hash(), r2.header().validators_hash());
    }
}
