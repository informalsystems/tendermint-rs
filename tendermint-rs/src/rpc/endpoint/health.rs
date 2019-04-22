//! `/health` endpoint JSONRPC wrapper

use crate::rpc;
use serde::{Deserialize, Serialize};

/// Perform a basic healthceck of the backend
#[derive(Default)]
pub struct Request;

impl rpc::Request for Request {
    type Response = Response;

    fn path(&self) -> rpc::request::Path {
        "/health".parse().unwrap()
    }
}

/// Healthcheck responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {}

impl rpc::Response for Response {}
