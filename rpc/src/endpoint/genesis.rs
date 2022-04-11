//! `/genesis` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};
use tendermint::Genesis;

/// Get the genesis state for the current chain
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request;

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::Genesis
    }
}

impl crate::SimpleRequest for Request {}

/// Block responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Genesis data
    pub genesis: Genesis,
}

impl crate::Response for Response {}
