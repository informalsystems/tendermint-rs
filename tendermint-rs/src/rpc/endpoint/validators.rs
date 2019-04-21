//! `/validators` endpoint JSONRPC wrapper

use crate::{block, rpc, validator};
use serde::{Deserialize, Serialize};

/// List validators for a specific block
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

    fn path(&self) -> rpc::request::Path {
        // TODO(tarcieri): use a `uri` crate to construct this?
        format!("/validators?height={}", self.height)
            .parse()
            .unwrap()
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
