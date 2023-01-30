//! `/health` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

use crate::dialect::Dialect;

/// Perform a basic healthceck of the backend
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request;

impl<S: Dialect> crate::Request<S> for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::Health
    }
}

impl<S: Dialect> crate::SimpleRequest<S> for Request {}

/// Healthcheck responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {}

impl crate::Response for Response {}
