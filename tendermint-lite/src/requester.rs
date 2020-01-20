use tendermint::block;
use tendermint::lite;
use tendermint::rpc;
use tendermint::validator;

use core::future::Future;
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

impl lite::types::Requester for RPCRequester {
    type SignedHeader = block::signed_header::SignedHeader;
    type ValidatorSet = validator::Set;

    /// Request the signed header at height h.
    /// If h==0, request the latest signed header.
    /// TODO: use an enum instead of h==0.
    fn signed_header<H>(&self, h: H) -> Result<Self::SignedHeader, lite::Error>
    where
        H: Into<block::Height>,
    {
        let height: block::Height = h.into();
        let r = match height.value() {
            0 => block_on(self.client.latest_commit()),
            _ => block_on(self.client.commit(height)),
        };
        match r {
            Ok(response) => Ok(response.signed_header),
            Err(_error) => Err(lite::Error::RequestFailed),
        }
    }

    /// Request the validator set at height h.
    fn validator_set<H>(&self, h: H) -> Result<Self::ValidatorSet, lite::Error>
    where
        H: Into<block::Height>,
    {
        let r = block_on(self.client.validators(h));
        match r {
            Ok(response) => Ok(validator::Set::new(response.validators)),
            Err(_error) => Err(lite::Error::RequestFailed),
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
    use tendermint::lite::types::SignedHeader as LiteSignedHeader;
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
