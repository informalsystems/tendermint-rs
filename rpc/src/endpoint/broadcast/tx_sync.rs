//! `/broadcast_tx_sync`: returns with the response from `CheckTx`.

use serde::{Deserialize, Serialize};

use tendermint::abci::{transaction, Code, Data, Log, Transaction};

/// `/broadcast_tx_sync`: returns with the response from `CheckTx`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Transaction to broadcast
    pub tx: Transaction,
}

impl Request {
    /// Create a new sync transaction broadcast RPC request
    pub fn new(tx: Transaction) -> Request {
        Request { tx }
    }
}

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::BroadcastTxSync
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
