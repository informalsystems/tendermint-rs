//! `/validators` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

use tendermint::{block, validator};

/// List validators for a specific block
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    pub height: block::Height,
}

impl Request {
    /// List validators for a specific block
    pub fn new(height: block::Height) -> Self {
        Self { height }
    }
}

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::Validators
    }
}

impl crate::SimpleRequest for Request {}

/// Validator responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Block height
    pub block_height: block::Height,

    /// Validator list
    pub validators: Vec<validator::Info>,
}

impl crate::Response for Response {}
