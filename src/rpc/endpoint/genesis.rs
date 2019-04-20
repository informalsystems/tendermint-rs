//! `/genesis` endpoint JSONRPC wrapper

use crate::{rpc, Genesis};
use serde::{Deserialize, Serialize};

/// Get the genesis state for the current chain
#[derive(Default)]
pub struct Request;

impl rpc::Request for Request {
    type Response = Response;

    fn path(&self) -> rpc::request::Path {
        "/genesis".parse().unwrap()
    }
}

/// Block responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Genesis data
    pub genesis: Genesis,
}

impl rpc::Response for Response {}
