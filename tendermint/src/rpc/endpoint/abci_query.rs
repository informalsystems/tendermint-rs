//! `/abci_query` endpoint JSONRPC wrapper

use crate::{
    abci::{Code, Log, Path, Proof},
    block, rpc, serializers,
};
use serde::{Deserialize, Serialize};

/// Query the ABCI application for information
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Path to the data
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<Path>,

    /// Data to query
    #[serde(
        serialize_with = "serializers::serialize_hex",
        deserialize_with = "serializers::parse_hex"
    )]
    data: Vec<u8>,

    /// Block height
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<block::Height>,

    /// Include proof in response
    prove: bool,
}

impl Request {
    /// Create a new ABCI query request
    pub fn new<D>(path: Option<Path>, data: D, height: Option<block::Height>, prove: bool) -> Self
    where
        D: Into<Vec<u8>>,
    {
        Self {
            path,
            data: data.into(),
            height,
            prove,
        }
    }
}

impl rpc::Request for Request {
    type Response = Response;

    fn method(&self) -> rpc::Method {
        rpc::Method::AbciQuery
    }
}

/// ABCI query response wrapper
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// ABCI query results
    pub response: AbciQuery,
}

impl rpc::Response for Response {}

/// ABCI query results
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct AbciQuery {
    /// Response code
    pub code: Code,

    /// Log value
    pub log: Log,

    /// Info value
    pub info: Option<String>,

    /// Index
    #[serde(
        serialize_with = "serializers::serialize_i64",
        deserialize_with = "serializers::parse_i64"
    )]
    pub index: i64,

    /// Key
    #[serde(
        default,
        serialize_with = "serializers::serialize_option_base64",
        deserialize_with = "serializers::parse_option_base64"
    )]
    pub key: Option<Vec<u8>>,

    /// Value
    #[serde(
        default,
        serialize_with = "serializers::serialize_option_base64",
        deserialize_with = "serializers::parse_option_base64"
    )]
    pub value: Option<Vec<u8>>,

    /// Proof (if requested)
    pub proof: Option<Proof>,

    /// Block height
    pub height: block::Height,

    /// Codespace
    pub codespace: Option<String>,
}
