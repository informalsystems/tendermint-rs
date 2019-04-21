//! `/commit` endpoint JSONRPC wrapper

use crate::{block, rpc};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// Get commit information about a specific block
pub struct Request {
    height: block::Height,
}

impl Request {
    /// Create a new request for commit info about a particular block
    pub fn new(height: block::Height) -> Self {
        Self { height }
    }
}

impl rpc::Request for Request {
    type Response = Response;

    fn path(&self) -> rpc::request::Path {
        // TODO(tarcieri): use a `uri` crate to construct this?
        format!("/commit?height={}", self.height).parse().unwrap()
    }
}

/// Commit responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Signed header
    pub signed_header: SignedHeader,

    /// Is the signed header canonical?
    pub canonical: bool,
}

impl rpc::Response for Response {}

/// Signed block headers
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignedHeader {
    /// Block header
    pub header: block::Header,
}

impl Deref for SignedHeader {
    type Target = block::Header;

    fn deref(&self) -> &block::Header {
        &self.header
    }
}
