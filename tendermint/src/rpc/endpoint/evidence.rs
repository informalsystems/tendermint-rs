
//! `/broadcast_evidence`: broadcast an evidence.

use crate::{
    abci::transaction,
    rpc,
    evidence::Evidence,
};
use serde::{Deserialize, Serialize};

/// `/broadcast_evidence`: broadcast an evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
    /// TODO: transaction::Hash should be tmhash (github.com/tendermint/tendermint/crypto/tmhash)
    pub hash: transaction::Hash,
}

impl rpc::Response for Response {}
