//! `/health` endpoint JSONRPC wrapper

use crate::rpc;
use serde::{Deserialize, Serialize};

/// Perform a basic healthceck of the backend
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request;

impl rpc::Request for Request {
    type Response = Response;

    fn method(&self) -> rpc::Method {
        rpc::Method::Health
    }
}

/// Healthcheck responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {}

impl rpc::Response for Response {}
