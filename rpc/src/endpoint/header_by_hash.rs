//! `/header_by_hash` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};
use tendermint::{block::Header, Hash};

/// Get information about a specific block by its hash
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Hash of the block to request.
    ///
    /// If no hash is provided, it will return no header (as if the hash
    /// did not match any block).
    ///
    /// Serialized internally into a hex-encoded string before sending to
    /// the RPC server.
    #[serde(default)]
    #[serde(with = "crate::serializers::option_hash")]
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
        crate::Method::HeaderByHash
    }
}

impl crate::SimpleRequest for Request {}

/// Header response
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Header data
    pub header: Option<Header>,
}

impl crate::Response for Response {}