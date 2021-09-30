//! `/subscribe` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Subscription request for events.
///
/// A subscription request is not a [`SimpleRequest`], because it does not
/// return a simple, singular response.
///
/// [`SimpleRequest`]: ../trait.SimpleRequest.html
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

impl crate::Response for Response {}
