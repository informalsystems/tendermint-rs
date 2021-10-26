//! `/abci_query` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

use tendermint::abci::{Code, Log, Path};
use tendermint::block;
use tendermint::merkle::proof::Proof;
use tendermint::serializers;

use crate::prelude::*;

/// Query the ABCI application for information
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    /// Path to the data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Path>,

    /// Data to query
    #[serde(with = "serializers::bytes::hexstring")]
    pub data: Vec<u8>,

    /// Block height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<block::Height>,

    /// Include proof in response
    #[serde(default)]
    pub prove: bool,
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

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::AbciQuery
    }
}

impl crate::SimpleRequest for Request {}

/// ABCI query response wrapper
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Response {
    /// ABCI query results
    pub response: AbciQuery,
}

impl crate::Response for Response {}

/// ABCI query results
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct AbciQuery {
    /// Response code
    pub code: Code,

    /// Log value
    pub log: Log,

    /// Info value
    #[serde(default = "String::new")]
    pub info: String,

    /// Index
    #[serde(with = "serializers::from_str")]
    pub index: i64,

    /// Key
    #[serde(default, with = "serializers::bytes::base64string")]
    pub key: Vec<u8>,

    /// Value
    #[serde(default, with = "serializers::bytes::base64string")]
    pub value: Vec<u8>,

    /// Proof (might be explicit null)
    #[serde(alias = "proofOps")]
    pub proof: Option<Proof>,

    /// Block height
    pub height: block::Height,

    /// Codespace
    #[serde(default = "String::new")]
    pub codespace: String,
}
