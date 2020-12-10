//! `/block` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

use tendermint::block::{self, Block};

/// Get information about a specific block
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Height of the block to request.
    ///
    /// If no height is provided, it will fetch results for the latest block.
    pub height: Option<block::Height>,
}

impl Request {
    /// Create a new request for information about a particular block
    pub fn new(height: block::Height) -> Self {
        Self {
            height: Some(height),
        }
    }
}

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::Block
    }
}

impl crate::SimpleRequest for Request {}

/// Block responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Block ID
    pub block_id: block::Id,

    /// Block data
    pub block: Block,
}

impl crate::Response for Response {}
