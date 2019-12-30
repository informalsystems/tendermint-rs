use tendermint::block;
use tendermint::lite;
use tendermint::rpc;
use tendermint::validator;

use core::future::Future;
use tokio::runtime::Builder;

pub struct Requester {
    client: rpc::Client,
}

impl Requester {
    fn new(client: rpc::Client) -> Self {
        Requester { client }
    }
}

impl lite::types::Requester for Requester {
    type SignedHeader = block::signed_header::SignedHeader;
    type ValidatorSet = validator::Set;

    /// Request the signed header at height h.
    fn signed_header<H>(&self, h: H) -> Result<Self::SignedHeader, lite::Error>
    where
        H: Into<block::Height>,
    {
        let r = block_on(self.client.commit(h));
        match r {
            Ok(response) => Ok(response.signed_header),
            Err(e) => Err(lite::Error::RequestFailed),
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
            Err(e) => Err(lite::Error::RequestFailed),
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
    use crate::*;
    use block::signed_header::SignedHeader;
    use tendermint::lite::types::Header as LiteHeader;
    use tendermint::lite::types::Requester as LiteRequester;
    use tendermint::lite::types::SignedHeader as LiteSignedHeader;
    use tendermint::lite::types::ValidatorSet as LiteValSet;
    use tendermint::net;
    use tendermint::rpc;
    use validator::Set;

    #[test]
    fn test_val_set() {
        let client = block_on(rpc::Client::new(&"localhost:26657".parse().unwrap())).unwrap();
        let req = Requester::new(client);
        let r = req.validator_set(5).unwrap();
        println!("{:?}", r.hash());
        let r = req.signed_header(5).unwrap();
        println!("{:?}", r.header().validators_hash());
    }
}
