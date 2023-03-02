//! `/broadcast_evidence`: broadcast an evidence.

use serde::{Deserialize, Serialize};
use tendermint::{evidence::Evidence, Hash};

use crate::{dialect::Dialect, request::RequestMessage, Method};

/// `/broadcast_evidence`: broadcast an evidence.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Request {
    /// Evidence to broadcast
    pub ev: Evidence,
}

impl Request {
    /// Create a new evidence broadcast RPC request
    pub fn new(ev: Evidence) -> Request {
        Request { ev }
    }
}

impl RequestMessage for Request {
    fn method(&self) -> Method {
        Method::BroadcastEvidence
    }
}

impl<S: Dialect> crate::Request<S> for Request {
    type Response = Response;
}

impl<S: Dialect> crate::SimpleRequest<S> for Request {
    type Output = Response;
}

/// Response from either an evidence broadcast request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Evidence hash
    pub hash: Hash,
}

impl crate::Response for Response {}
