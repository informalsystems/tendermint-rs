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
    pub code: Code,

    /// Log value
    pub log: Log,

    /// Info value
    pub info: Option<String>,

    /// Index
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "serializers::serialize_i64",
            deserialize_with = "serializers::parse_i64"
        )
    )]
    pub index: i64,

    /// Key
    // TODO(tarcieri): parse to Vec<u8>?
    pub key: String,

    /// Value
    // TODO(tarcieri): parse to Vec<u8>?
    pub value: String,

    /// Proof (if requested)
    pub proof: Option<Proof>,

    /// Block height
    pub height: block::Height,

    /// Codespace
    pub codespace: Option<String>,
}
