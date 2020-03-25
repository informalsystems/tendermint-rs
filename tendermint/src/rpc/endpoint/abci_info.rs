//! `/abci_info` endpoint JSONRPC wrapper

use crate::{block, rpc, version};
use serde::{
    Deserialize,
    Serialize,
};
use crate::serializers;

/// Request ABCI information from a node
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request;

impl rpc::Request for Request {
    type Response = Response;

    fn method(&self) -> rpc::Method {
        rpc::Method::AbciInfo
    }
}

/// ABCI information response
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// ABCI info
    pub response: AbciInfo,
}

impl rpc::Response for Response {}

/// ABCI information
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct AbciInfo {
    /// Name of the application
    pub data: String,

    /// Version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// App version, omit empty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_version: Option<version::Protocol>,

    /// Last block height, omit empty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_block_height: Option<block::Height>,

    /// Last app hash for the block, omit empty
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "serializers::parse_base64",
        serialize_with = "serializers::serialize_base64",
    )]
    pub last_block_app_hash: Vec<u8>,
}

/// Default trait implements default values for the optional last_block_height and last_block_app_hash
/// for cases where they were omitted from the JSON.
impl Default for AbciInfo {
    fn default() -> Self {
        AbciInfo {
            data: "".to_string(),
            version: None,
            app_version: None,
            last_block_height: None,
            last_block_app_hash: Vec::from(""),
        }
    }
}
