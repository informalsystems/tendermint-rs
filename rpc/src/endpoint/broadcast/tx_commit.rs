//! `/broadcast_tx_commit`: only returns error if `mempool.CheckTx()` errs or
//! if we timeout waiting for tx to commit.

use serde::{Deserialize, Serialize};
use tendermint::{abci, block, Hash};

use crate::{prelude::*, serializers};

/// `/broadcast_tx_commit`: only returns error if `mempool.CheckTx()` errs or
/// if we timeout waiting for tx to commit.
///
/// If `CheckTx` or `DeliverTx` fail, no error will be returned, but the
/// returned result will contain a non-OK ABCI code.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Transaction to broadcast
    #[serde(with = "serializers::bytes::base64string")]
    pub tx: Vec<u8>,
}

impl Request {
    /// Create a new commit transaction broadcast RPC request
    pub fn new(tx: impl Into<Vec<u8>>) -> Request {
        Request { tx: tx.into() }
    }
}

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::BroadcastTxCommit
    }
}

impl crate::SimpleRequest for Request {}

/// Response from `/broadcast_tx_commit`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// `CheckTx` result
    pub check_tx: abci::response::CheckTx,

    /// `DeliverTx` result
    pub deliver_tx: abci::response::DeliverTx,

    /// Transaction
    pub hash: Hash,

    /// Height
    pub height: block::Height,
}

impl crate::Response for Response {}
