
//! `/broadcast_evidence`: broadcast an evidence.

use crate::{
    abci::{transaction},
    rpc,
    evidence::Evidence,
};
use serde::{Deserialize, Serialize};

/// `/broadcast_evidence`: broadcast an evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Evidence to broadcast
    pub e: Evidence,
}

impl Request {
    /// Create a new evidence broadcast RPC request
    pub fn new(e: Evidence) -> Request {
        Request { e }
    }
}

impl rpc::Request for Request {
    type Response = Response;

    fn method(&self) -> rpc::Method {
        rpc::Method::BroadcastEvidence
    }
}

/// Response from either an evidence broadcast request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Evidence hash
    pub hash: transaction::Hash,
}

impl rpc::Response for Response {}
