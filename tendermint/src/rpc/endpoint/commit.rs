//! `/commit` endpoint JSONRPC wrapper

use crate::{block, block::signed_header::SignedHeader, rpc};
use serde::{Deserialize, Serialize};

/// Get commit information about a specific block
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    height: Option<block::Height>,
}

impl Request {
    /// Create a new request for commit info about a particular block
    pub fn new(height: block::Height) -> Self {
        let mut height_opt = Some(height);
        if height.value() == 0 {
            height_opt = None
        }
        Self { height: height_opt }
    }
}

impl rpc::Request for Request {
    type Response = Response;

    fn method(&self) -> rpc::Method {
        rpc::Method::Commit
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
