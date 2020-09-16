//! `/abci_info` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

use tendermint::block;
use tendermint::serializers;

/// Request ABCI information from a node
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request;

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::AbciInfo
    }
}

/// ABCI information response
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// ABCI info
    pub response: AbciInfo,
}

impl crate::Response for Response {}

/// ABCI information
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct AbciInfo {
    /// Name of the application
    pub data: String,

    /// Version
    pub version: String,

    /// App version
    #[serde(with = "serializers::from_str")]
    pub app_version: u64,

    /// Last block height
    pub last_block_height: block::Height,

    /// Last app hash for the block
    #[serde(skip_serializing_if = "Vec::is_empty", with = "serde_bytes")]
    pub last_block_app_hash: Vec<u8>,
}
