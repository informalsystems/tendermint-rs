//! `/block_by_hash` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};
use tendermint::{
    block::{self, Block},
    Hash,
};

use crate::{dialect::Dialect, request::RequestMessage};

/// Get information about a specific block by its hash
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Hash of the block to request.
    ///
    /// If no hash is provided, it will return no block (as if the hash
    /// did not match any block).
    ///
    /// Serialized internally into a base64-encoded string before sending to
    /// the RPC server.
    #[serde(default)]
    #[serde(with = "crate::serializers::opt_tm_hash_base64")]
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

impl RequestMessage for Request {
    fn method(&self) -> crate::Method {
        crate::Method::BlockByHash
    }
}

impl<S: Dialect> crate::Request<S> for Request {
    type Response = Response;
}

impl<S: Dialect> crate::SimpleRequest<S> for Request {
    type Output = Response;
}

/// Block responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Block ID
    pub block_id: block::Id,

    /// Block data
    pub block: Option<Block>,
}

impl crate::Response for Response {}
