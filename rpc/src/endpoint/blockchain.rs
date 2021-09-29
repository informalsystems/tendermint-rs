//! `/block` endpoint JSON-RPC wrapper

use core::ops::Range;
use serde::{Deserialize, Serialize};

use tendermint::block;

use crate::prelude::*;

/// Get information about a specific block
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// First block in the sequence to request info about
    #[serde(rename = "minHeight")]
    pub min_height: block::Height,

    /// Last block in the sequence to request info about
    #[serde(rename = "maxHeight")]
    pub max_height: block::Height,
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

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::Blockchain
    }
}

impl crate::SimpleRequest for Request {}

/// Block responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Last block height for this particular chain
    pub last_height: block::Height,

    /// Block metadata
    pub block_metas: Vec<block::Meta>,
}

impl crate::Response for Response {}
