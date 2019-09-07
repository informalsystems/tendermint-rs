//! `/genesis` endpoint JSONRPC wrapper

use crate::{rpc, Genesis};
use serde::{Deserialize, Serialize};

/// Get the genesis state for the current chain
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request;

impl rpc::Request for Request {
    type Response = Response;

    fn method(&self) -> rpc::Method {
        rpc::Method::Genesis
    }
}

/// Block responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Genesis data
    pub genesis: Genesis,
}

impl rpc::Response for Response {}
