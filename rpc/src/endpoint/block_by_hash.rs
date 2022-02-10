//! `/block` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

use tendermint::Hash;
use tendermint::block::{self, Block};

/// Get information about a specific block
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Hash of the block to request.
    ///
    /// If no hash is provided, it will fetch results for the latest block.
    pub hash: Option<Hash>,
}

impl Request {
    /// Create a new request for information about a particular block
    pub fn new(hash: Hash) -> Self {
        Self {
            hash: Some(hash),
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
