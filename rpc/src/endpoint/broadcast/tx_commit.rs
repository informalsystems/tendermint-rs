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
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct TxResult {
    /// Code
    pub code: Code,

    /// Data
    #[serde(with = "tendermint_proto::serializers::optional")]
    pub data: Option<Data>,

    /// Log
    #[serde(default)]
    pub log: Log,

    /// ABCI info (nondeterministic)
    #[serde(default)]
    pub info: Info,

    /// Amount of gas wanted
    #[serde(default)]
    pub gas_wanted: Gas,

    /// Amount of gas used
    #[serde(default)]
    pub gas_used: Gas,

    /// Events
    #[serde(default)]
    pub events: Vec<Event>,

    /// Codespace
    #[serde(default)]
    pub codespace: Codespace,
}
