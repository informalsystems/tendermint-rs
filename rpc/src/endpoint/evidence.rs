//! `/broadcast_evidence`: broadcast an evidence.

use crate::Method;

use serde::{Deserialize, Serialize};
use tendermint::{abci::transaction, evidence::Evidence};

/// `/broadcast_evidence`: broadcast an evidence.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
    /// TODO: transaction::Hash should be tmhash (github.com/tendermint/tendermint/crypto/tmhash)
    pub hash: transaction::Hash,
}

impl crate::Response for Response {}
