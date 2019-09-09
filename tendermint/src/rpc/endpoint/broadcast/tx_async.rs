//! `/broadcast_tx_async`: broadcast a transaction and return immediately.

use crate::{
    abci::{transaction, Code, Data, Log, Transaction},
    rpc,
};
use serde::{Deserialize, Serialize};

/// `/broadcast_tx_async`: broadcast a transaction and return immediately.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Transaction to broadcast
    pub tx: Transaction,
}

impl Request {
    /// Create a new async transaction broadcast RPC request
    pub fn new(tx: Transaction) -> Request {
        Request { tx }
    }
}

impl rpc::Request for Request {
    type Response = Response;

    fn method(&self) -> rpc::Method {
        rpc::Method::BroadcastTxAsync
    }
}

/// Response from either an async or sync transaction broadcast request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Code
    pub code: Code,

    /// Data
    pub data: Data,

    /// Log
    pub log: Log,

    /// Transaction hash
    pub hash: transaction::Hash,
}

impl rpc::Response for Response {}
