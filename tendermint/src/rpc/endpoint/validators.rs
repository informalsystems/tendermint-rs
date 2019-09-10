//! `/validators` endpoint JSONRPC wrapper

use crate::{block, rpc, validator};
use serde::{Deserialize, Serialize};

/// List validators for a specific block
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    height: block::Height,
}

impl Request {
    /// List validators for a specific block
    pub fn new(height: block::Height) -> Self {
        Self { height }
    }
}

impl rpc::Request for Request {
    type Response = Response;

    fn method(&self) -> rpc::Method {
        rpc::Method::Validators
    }
}

/// Validator responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Block height
    pub block_height: block::Height,

    /// Validator list
    pub validators: Vec<validator::Info>,
}

impl rpc::Response for Response {}
