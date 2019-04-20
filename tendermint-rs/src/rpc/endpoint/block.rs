//! `/block` endpoint JSONRPC wrapper

use crate::{
    block::{self, Block},
    rpc,
};
use serde::{Deserialize, Serialize};

/// Get information about a specific block
pub struct Request {
    height: block::Height,
}

impl Request {
    /// Create a new request for information about a particular block
    pub fn new<H>(height: block::Height) -> Self {
        Self { height }
    }
}

impl rpc::Request for Request {
    type Response = Response;

    fn path(&self) -> rpc::request::Path {
        // TODO(tarcieri): use a `uri` crate to construct this?
        format!("/block?height={}", self.height).parse().unwrap()
    }
}

/// Block responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Block metadata
    pub block_meta: block::Meta,

    /// Block data
    pub block: Block,
}

impl rpc::Response for Response {}
