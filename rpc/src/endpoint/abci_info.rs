//! `/abci_info` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

use crate::dialect::Dialect;
use crate::request::RequestMessage;

/// Request ABCI information from a node
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request;

impl RequestMessage for Request {
    fn method(&self) -> crate::Method {
        crate::Method::AbciInfo
    }
}

impl<S: Dialect> crate::Request<S> for Request {
    type Response = Response;
}

impl<S: Dialect> crate::SimpleRequest<S> for Request {}

/// ABCI information response
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// ABCI info
    pub response: tendermint::abci::response::Info,
}

impl crate::Response for Response {}
