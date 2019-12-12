//! `/abci_info` endpoint JSONRPC wrapper

use crate::{block, rpc, Hash};
use crate::{hash, serializers};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

use subtle_encoding::base64;

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
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct AbciInfo {
    /// Name of the application
    pub data: String,

    /// Version
    pub version: Option<String>,

    /// App version
    #[serde(
        serialize_with = "serializers::serialize_u64",
        deserialize_with = "serializers::parse_u64"
    )]
    pub app_version: u64,

    /// Last block height
    pub last_block_height: Option<block::Height>,

    /// Last app hash for the block
    #[serde(
        serialize_with = "serialize_app_hash",
        deserialize_with = "parse_app_hash"
    )]
    pub last_block_app_hash: Hash,
}

/// Parse Base64-encoded app hash
pub(crate) fn parse_app_hash<'de, D>(deserializer: D) -> Result<Hash, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes = base64::decode(String::deserialize(deserializer)?.as_bytes())
        .map_err(|e| D::Error::custom(format!("{}", e)))?;

    Hash::new(hash::Algorithm::Sha256, &bytes).map_err(|e| D::Error::custom(format!("{}", e)))
}

/// Serialize Base64-encoded app hash
pub(crate) fn serialize_app_hash<S>(hash: &Hash, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    hash.as_bytes()
        .map(|bytes| String::from_utf8(base64::encode(bytes)).unwrap())
        .unwrap_or_default()
        .serialize(serializer)
}
