//! `/broadcast_tx_async`: broadcast a transaction and return immediately.

use crate::serializers;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tendermint::Hash;

use crate::abci::{Code, Log, Transaction};

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
    #[serde(with = "serializers::bytes::base64string")]
    pub data: Bytes,

    /// Log
    pub log: Log,

    /// Transaction hash
    pub hash: Hash,
}

impl crate::Response for Response {}
