//! `/header` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

use tendermint::block::{Header, Height};

/// Get information about a specific header
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Height of the header to request.
    ///
    /// If no height is provided, it will fetch results for the latest header.
    pub height: Option<Height>,
}

impl Request {
    /// Create a new request for information about a particular header
    pub fn new(height: Height) -> Self {
        Self {
            height: Some(height),
        }
    }
}

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::Header
    }
}

impl crate::SimpleRequest for Request {}

/// Header responses
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Response {
    /// Header
    pub header: Header,
}

impl crate::Response for Response {}
