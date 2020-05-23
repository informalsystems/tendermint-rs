//! `/subscribe` endpoint JSONRPC wrapper

use crate::rpc;
use serde::{Deserialize, Serialize};
use std::io::Read;

/// Subscribe request for events on websocket
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    query: String,
}

impl Request {
    /// Query the Tendermint nodes event and stream events over web socket
    pub fn new(query: String) -> Self {
        Self { query }
    }
}

impl rpc::Request for Request {
    type Response = Response;

    fn method(&self) -> rpc::Method {
        rpc::Method::Subscribe
    }
}

/// Status responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {}

/// Subcribe is weird RPC endpoint. It's only meaningful at websocket response and there isn't a
/// synchronous reponse offered. It there is an error it's asynchronous and we don't try and stich
/// the async response backer together with the request.
impl rpc::Response for Response {
    /// We throw away response data JSON string so swallow errors and return the empty Response
    fn from_string(_response: impl AsRef<[u8]>) -> Result<Self, rpc::Error> {
        Ok(Response {})
    }

    /// We throw away responses in `subscribe` to swallow errors from the `io::Reader` and provide
    /// the Response
    fn from_reader(_reader: impl Read) -> Result<Self, rpc::Error> {
        Ok(Response {})
    }
}
