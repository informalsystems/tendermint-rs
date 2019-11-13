//! `/commit` endpoint JSONRPC wrapper

use crate::vote::SignedVote;
use crate::{block, lite, rpc, Hash};
use serde::{Deserialize, Serialize};

/// Get commit information about a specific block
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    height: Option<block::Height>,
}

impl Request {
    /// Create a new request for commit info about a particular block
    pub fn new(height: block::Height) -> Self {
        Self {
            height: Some(height),
        }
    }
}

impl rpc::Request for Request {
    type Response = Response;

    fn method(&self) -> rpc::Method {
        rpc::Method::Commit
    }
}

/// Commit responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Signed header
    pub signed_header: SignedHeader,

    /// Is the signed header canonical?
    pub canonical: bool,
}

impl rpc::Response for Response {}

/// Signed block headers
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignedHeader {
    /// Block header
    pub header: block::Header,
    /// Commit containing signatures for the header
    pub commit: block::Commit,
}

impl lite::Commit for SignedHeader {
    type Vote = SignedVote;
    fn header_hash(&self) -> Hash {
        self.commit.block_id.hash
    }
    fn into_vec(&self) -> Vec<Option<Self::Vote>> {
        let chain_id = self.header.chain_id.to_string();
        let mut votes = self.commit.precommits.clone().into_vec();
        votes
            .drain(..)
            .map(|opt| {
                opt.map(|vote| {
                    SignedVote::new(
                        (&vote).into(),
                        &chain_id,
                        vote.validator_address,
                        vote.signature,
                    )
                })
            })
            .collect()
    }
}
