//! `/header` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};
use tendermint::block::{self, Header};

use crate::request::RequestMessage;
use crate::v0_37;

/// Get information about a specific block
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Height of the block header to request.
    ///
    /// If no height is provided, it will fetch results for the latest block.
    pub height: Option<block::Height>,
}

impl Request {
    /// Create a new request for header information about a particular block
    pub fn new(height: block::Height) -> Self {
        Self {
            height: Some(height),
        }
    }
}

impl RequestMessage for Request {
    fn method(&self) -> crate::Method {
        crate::Method::Header
    }
}

impl crate::Request<v0_37::Dialect> for Request {
    type Response = Response;
}

impl crate::SimpleRequest<v0_37::Dialect> for Request {
    type Output = Response;
}

/// Header response
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Header data
    pub header: Header,
}

impl crate::Response for Response {}

impl From<super::block::Response> for Response {
    fn from(block_resp: super::block::Response) -> Self {
        Response {
            header: block_resp.block.header,
        }
    }
}
