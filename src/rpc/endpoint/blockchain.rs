//! `/block` endpoint JSONRPC wrapper

use crate::{block, rpc};
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Get information about a specific block
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// First block in the sequence to request info about
    #[serde(rename = "minHeight")]
    min_height: block::Height,

    /// Last block in the sequence to request info about
    #[serde(rename = "maxHeight")]
    max_height: block::Height,
}

impl Request {
    /// Request information about a sequence of blocks
    pub fn new(min_height: block::Height, max_height: block::Height) -> Self {
        Self {
            min_height,
            max_height,
        }
    }
}

impl From<Range<block::Height>> for Request {
    fn from(range: Range<block::Height>) -> Request {
        Request::new(range.start, range.end)
    }
}

impl rpc::Request for Request {
    type Response = Response;

    fn method(&self) -> rpc::Method {
        rpc::Method::Blockchain
    }
}

/// Block responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Last block height for this particular chain
    pub last_height: block::Height,

    /// Block metadata
    #[serde(default)]
    pub block_metas: Vec<block::Meta>,
}

impl rpc::Response for Response {}
