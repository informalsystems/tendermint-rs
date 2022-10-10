//! `/broadcast_tx_sync`: returns with the response from `CheckTx`.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tendermint::{abci::Code, Hash};

use crate::{prelude::*, serializers};

/// `/broadcast_tx_sync`: returns with the response from `CheckTx`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Transaction to broadcast
    #[serde(with = "serializers::bytes::base64string")]
    pub tx: Vec<u8>,
}

impl Request {
    /// Create a new sync transaction broadcast RPC request
    pub fn new(tx: impl Into<Vec<u8>>) -> Request {
        Request { tx: tx.into() }
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
    #[serde(with = "serializers::bytes::base64string")]
    pub data: Bytes,

    /// Log
    pub log: String,

    /// Transaction hash
    pub hash: Hash,
}

impl crate::Response for Response {}
