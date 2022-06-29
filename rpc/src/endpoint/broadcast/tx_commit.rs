//! `/broadcast_tx_commit`: only returns error if `mempool.CheckTx()` errs or
//! if we timeout waiting for tx to commit.

use serde::{Deserialize, Serialize};

use tendermint::abci::responses::Codespace;
use tendermint::abci::{Event, Gas, Info};
use tendermint::{
    abci::{transaction, Code, Data, Log, Transaction},
    block,
};

use crate::prelude::*;

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
    pub hash: transaction::Hash,

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
    #[serde(with = "tendermint_proto::serializers::optional")]
    pub data: Option<Data>,

    /// Log
    pub log: Log,

    /// ABCI info (nondeterministic)
    pub info: Info,

    /// Amount of gas wanted
    pub gas_wanted: Gas,

    /// Amount of gas used
    pub gas_used: Gas,

    /// Events
    pub events: Vec<Event>,

    /// Codespace
    pub codespace: Codespace,

    /// Only relevant for `CheckTx`.
    pub sender: String,

    /// If the prioritized mempool is enabled, this will give an indication as
    /// to the priority assigned to the transaction.
    ///
    /// Only relevant for `CheckTx`.
    #[serde(with = "tendermint_proto::serializers::from_str")]
    pub priority: i64,

    /// Only relevant for `CheckTx`.
    pub mempool_error: String,
}
