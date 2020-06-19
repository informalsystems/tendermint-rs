//! `/broadcast_evidence`: broadcast an evidence.

use crate::Method;
use crate::Request as RpcRequest;
use crate::Response as RpcResponse;

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

impl RpcRequest for Request {
    type Response = Response;

    fn method(&self) -> Method {
        Method::BroadcastEvidence
    }
}

/// Response from either an evidence broadcast request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Evidence hash
    /// TODO: transaction::Hash should be tmhash (github.com/tendermint/tendermint/crypto/tmhash)
    pub hash: transaction::Hash,
}

impl RpcResponse for Response {}
