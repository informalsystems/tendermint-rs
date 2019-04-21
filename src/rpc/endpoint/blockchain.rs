//! `/block` endpoint JSONRPC wrapper

use crate::{block, rpc};
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Get information about a specific block
pub struct Request {
    /// First block in the sequence to request info about
    min: block::Height,

    /// Last block in the sequence to request info about
    max: block::Height,
}

impl Request {
    /// Request information about a sequence of blocks
    pub fn new(min: block::Height, max: block::Height) -> Self {
        Self { min, max }
    }
}

impl From<Range<block::Height>> for Request {
    fn from(range: Range<block::Height>) -> Request {
        Request::new(range.start, range.end)
    }
}

impl rpc::Request for Request {
    type Response = Response;

    fn path(&self) -> rpc::request::Path {
        // TODO(tarcieri): use a `uri` crate to construct this?
        format!("/block?minHeight={}&maxHeight={}", self.min, self.max)
            .parse()
            .unwrap()
    }
}

/// Block responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Last block height for this particular chain
    pub last_height: block::Height,

    /// Block metadata
    pub block_metas: Vec<block::Meta>,
}

impl rpc::Response for Response {}
