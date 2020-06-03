//! `/broadcast_tx_commit`: only returns error if `mempool.CheckTx()` errs or
//! if we timeout waiting for tx to commit.

use crate::{
    abci::{transaction, Code, Data, Log, Transaction},
    block, rpc,
};
use serde::{Deserialize, Serialize};

/// `/broadcast_tx_commit`: only returns error if `mempool.CheckTx()` errs or
/// if we timeout waiting for tx to commit.
///
/// If `CheckTx` or `DeliverTx` fail, no error will be returned, but the
/// returned result will contain a non-OK ABCI code.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Transaction to broadcast
    pub tx: Transaction,
}

impl Request {
    /// Create a new commit transaction broadcast RPC request
    pub fn new(tx: Transaction) -> Request {
        Request { tx }
    }
}

impl rpc::Request for Request {
    type Response = Response;

    fn method(&self) -> rpc::Method {
        rpc::Method::BroadcastTxCommit
    }
}

/// Response from `/broadcast_tx_commit`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// `CheckTx` result
    pub check_tx: TxResult,

    /// `DeliverTx` result
    pub deliver_tx: TxResult,

    /// Transaction
    pub hash: transaction::Hash,

    /// Height
    pub height: block::Height,
}

impl rpc::Response for Response {}

/// Results from either `CheckTx` or `DeliverTx`.
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct TxResult {
    /// Code
    pub code: Code,

    /// Data
    pub data: Option<Data>,

    /// Log
    pub log: Log,
}
