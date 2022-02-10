//! `/block_by_hash` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

use tendermint::Hash;
use tendermint::block::{self, Block};

/// Get information about a specific block by its hash
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Hash of the block to request.
    ///
    /// If no hash is provided, it will return no block (as if the hash
    /// did not match any block).
    pub hash: Option<Hash>,
}

impl Request {
    /// Create a new request for information about a particular block
    pub fn new<H: Into<Hash>>(hash: H) -> Self {
        Self {
            hash: Some(hash.into()),
        }
    }
}

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::BlockByHash
    }
}

impl crate::SimpleRequest for Request {}

/// Block responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Block ID
    pub block_id: block::Id,

    /// Block data
    pub block: Option<Block>,
}

impl crate::Response for Response {}
