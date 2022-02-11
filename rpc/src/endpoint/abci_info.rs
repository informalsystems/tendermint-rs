//! `/abci_info` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

/// Request ABCI information from a node
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request;

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::AbciInfo
    }
}

impl crate::SimpleRequest for Request {}

/// ABCI information response
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// ABCI info
    pub response: tendermint::abci::response::Info,
}

impl crate::Response for Response {}
