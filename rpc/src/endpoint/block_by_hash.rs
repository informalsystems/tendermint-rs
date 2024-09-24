//! `/block_by_hash` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};
use tendermint::{
    block::{self, Block},
    Hash,
};

use crate::dialect::{self, Dialect};
use crate::request::RequestMessage;

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

impl crate::Request<dialect::v0_34::Dialect> for Request {
    type Response = Response;
}

impl crate::Request<dialect::v0_37::Dialect> for Request {
    type Response = Response;
}

impl crate::Request<dialect::v0_38::Dialect> for Request {
    type Response = self::v0_38::DialectResponse;
}

impl<S: Dialect> crate::SimpleRequest<S> for Request
where
    Self: crate::Request<S>,
    Response: From<Self::Response>,
{
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

pub mod v0_38 {
    use super::*;
    use crate::endpoint::block::v0_38::DialectBlock;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct DialectResponse {
        /// Block ID
        pub block_id: block::Id,

        /// Block data
        pub block: Option<DialectBlock>,
    }

    impl crate::Response for DialectResponse {}

    impl From<DialectResponse> for Response {
        fn from(msg: DialectResponse) -> Self {
            Self {
                block_id: msg.block_id,
                block: msg.block.map(Into::into),
            }
        }
    }
}
