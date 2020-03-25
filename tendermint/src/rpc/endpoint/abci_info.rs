//! `/abci_info` endpoint JSONRPC wrapper

use crate::{block, rpc, version};
use serde::{
    // de::Error as _,
    Deserialize,
    // Deserializer,
    Serialize,
    // Serializer
};
// use subtle_encoding::base64;

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
    pub last_block_app_hash: Option<Vec<u8>>,
}

/// Parse Base64-encoded app hash
// pub(crate) fn parse_app_hash<'de, D>(deserializer: D) -> Result<Option<Hash>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let bytes = base64::decode(String::deserialize(deserializer)?.as_bytes())
//         .map_err(|e| D::Error::custom(format!("{}", e)))?;
//
//     Hash::new(hash::Algorithm::Sha256, &bytes) // This never returns None
//         .map(Some) // Return Option<Hash> (syntactic sugar so the value can be omitted in the struct)
//         .map_err(|e| D::Error::custom(format!("{}", e))) // or return custom Error
// }
//
// /// Serialize Base64-encoded app hash
// pub(crate) fn serialize_app_hash<S>(hash: &Option<Hash>, serializer: S) -> Result<S::Ok, S::Error>
// where
//     S: Serializer,
// {
//     String::from_utf8(base64::encode(hash.unwrap().as_bytes()))
//         .unwrap()
//         .serialize(serializer)
// }

/// Default trait implements default values for the optional last_block_height and last_block_app_hash
/// for cases where they were omitted from the JSON.
impl Default for AbciInfo {
    fn default() -> Self {
        AbciInfo {
            data: "".to_string(),
            version: None,
            app_version: None,
            last_block_height: None,
            last_block_app_hash: None,
        }
    }
}
