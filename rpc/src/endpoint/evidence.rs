//! `/broadcast_evidence`: broadcast an evidence.

use serde::{Deserialize, Serialize};
use tendermint::{evidence::Evidence, Hash};

use crate::Method;

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

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> Method {
        Method::BroadcastEvidence
    }
}

impl crate::SimpleRequest for Request {}

/// Response from either an evidence broadcast request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Evidence hash
    pub hash: Hash,
}

impl crate::Response for Response {}
