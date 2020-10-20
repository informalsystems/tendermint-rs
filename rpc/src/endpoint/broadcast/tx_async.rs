//! `/broadcast_tx_async`: broadcast a transaction and return immediately.

use serde::{Deserialize, Serialize};

use tendermint::abci::{transaction, Code, Data, Log, Transaction};

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

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::BroadcastTxAsync
    }
}

impl crate::SimpleRequest for Request {}

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

impl crate::Response for Response {}
