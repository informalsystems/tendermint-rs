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
    path: Option<Path>,

    /// Data to query
    data: Vec<u8>,

    /// Block height
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AbciQuery {
    /// Response code
    #[serde(default)]
    pub code: Code,

    /// Log value
    #[serde(default)]
    pub log: Log,

    /// Info value
    pub info: Option<String>,

    /// Index
    #[serde(
        serialize_with = "serializers::serialize_i64",
        deserialize_with = "serializers::parse_i64",
        default
    )]
    pub index: i64,

    /// Key
    // TODO(tarcieri): parse to Vec<u8>?
    pub key: Option<String>,

    /// Value
    // TODO(tarcieri): parse to Vec<u8>?
    pub value: Option<String>,

    /// Proof (if requested)
    pub proof: Option<Proof>,

    /// Block height
    #[serde(default, deserialize_with = "serializers::parse_height_option")]
    pub height: Option<block::Height>,

    /// Codespace
    pub codespace: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn parse_query_responses() {
        let error_rsp = r#"{"response": {"code": 1, "log": "account lookup failed: account not found", "info": "", "index": "0", "key": null, "value": null, "proof": null, "height": "0", "codespace": ""}}"#;
        let success_rsp = r#"{"response": {"value": "some value"}}"#;
        serde_json::from_str::<Response>(error_rsp).expect("parse error response");
        serde_json::from_str::<Response>(success_rsp).expect("parse success response");
    }
}
