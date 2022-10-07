//! `/broadcast_tx_commit`: only returns error if `mempool.CheckTx()` errs or
//! if we timeout waiting for tx to commit.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tendermint::{abci, block, Hash};

use crate::{
    abci::{Code, Transaction},
    prelude::*,
    serializers,
};

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
    pub check_tx: TxResult,

    /// `DeliverTx` result
    pub deliver_tx: TxResult,

    /// Transaction
    pub hash: Hash,

    /// Height
    pub height: block::Height,
}

impl crate::Response for Response {}

/// Results from either `CheckTx` or `DeliverTx`.
///
/// Prioritized mempool-related fields are only relevant for `CheckTx` results.
/// The results for `CheckTx` and `DeliverTx` are not separated in tendermint-rs
/// v0.23.x to avoid breaking the API, as Tendermint v0.34.0-v0.34.19 returned
/// the exact same result structure, and this changed in v0.34.20.
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct TxResult {
    /// Code
    pub code: Code,

    /// Data
    #[serde(with = "tendermint::serializers::bytes::base64string")]
    pub data: Bytes,

    /// Log
    pub log: String,

    /// ABCI info (nondeterministic)
    pub info: String,

    /// Amount of gas wanted
    #[serde(with = "serializers::from_str")]
    pub gas_wanted: i64,

    /// Amount of gas used
    #[serde(with = "serializers::from_str")]
    pub gas_used: i64,

    /// Events
    pub events: Vec<abci::Event>,

    /// Codespace
    pub codespace: String,

    /// Only relevant for `CheckTx`.
    pub sender: String,

    /// If the prioritized mempool is enabled, this will give an indication as
    /// to the priority assigned to the transaction.
    ///
    /// Only relevant for `CheckTx`.
    #[serde(with = "serializers::from_str")]
    pub priority: i64,

    /// Only relevant for `CheckTx`.
    pub mempool_error: String,
}
