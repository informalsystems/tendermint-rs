//! `/subscribe` endpoint JSONRPC wrapper

use serde::{Deserialize, Serialize};
use std::io::Read;

/// Subscription request for events.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    pub query: String,
}

impl Request {
    /// Query the Tendermint nodes event and stream events (by default over a
    /// WebSocket connection).
    pub fn new(query: String) -> Self {
        Self { query }
    }
}

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::Subscribe
    }
}

/// Status responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {}

/// Subscribe does not have a meaningful response at the moment.
impl crate::Response for Response {
    /// We throw away response data JSON string so swallow errors and return the empty Response
    fn from_string(_response: impl AsRef<[u8]>) -> Result<Self, crate::Error> {
        Ok(Response {})
    }

    /// We throw away responses in `subscribe` to swallow errors from the `io::Reader` and provide
    /// the Response
    fn from_reader(_reader: impl Read) -> Result<Self, crate::Error> {
        Ok(Response {})
    }
}
