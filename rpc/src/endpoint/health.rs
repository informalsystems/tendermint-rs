//! `/health` endpoint JSONRPC wrapper

use serde::{Deserialize, Serialize};

/// Perform a basic healthceck of the backend
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request;

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::Health
    }
}

/// Healthcheck responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {}

impl crate::Response for Response {}
