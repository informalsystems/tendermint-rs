use bytes::Bytes;
use serde::{Deserialize, Serialize};

use tendermint::abci::{self, Code};

use crate::prelude::*;
use crate::serializers;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CheckTx<Ev> {
    /// The response code.
    ///
    /// Transactions where `code != 0` will be rejected; these transactions will
    /// not be broadcast to other nodes or included in a proposal block.
    /// Tendermint attributes no other value to the response code.
    pub code: Code,
    /// Result bytes, if any.
    #[serde(with = "serializers::nullable")]
    pub data: Bytes,
    /// The output of the application's logger.
    ///
    /// **May be non-deterministic**.
    pub log: String,
    /// Additional information.
    ///
    /// **May be non-deterministic**.
    pub info: String,
    /// Amount of gas requested for the transaction.
    #[serde(with = "serializers::from_str")]
    pub gas_wanted: i64,
    /// Amount of gas consumed by the transaction.
    #[serde(with = "serializers::from_str")]
    pub gas_used: i64,
    /// Events that occurred while checking the transaction.
    pub events: Vec<Ev>,
    /// The namespace for the `code`.
    pub codespace: String,
    /// The transaction's sender (e.g. the signer).
    pub sender: String,
    /// The transaction's priority (for mempool ordering).
    #[serde(with = "serializers::from_str")]
    pub priority: i64,
    /// mempool_error is set by Tendermint.
    /// ABCI applictions should not set mempool_error.
    pub mempool_error: String,
}

impl<Ev> Default for CheckTx<Ev> {
    fn default() -> Self {
        Self {
            code: Default::default(),
            data: Default::default(),
            log: Default::default(),
            info: Default::default(),
            gas_wanted: Default::default(),
            gas_used: Default::default(),
            events: Default::default(),
            codespace: Default::default(),
            sender: Default::default(),
            priority: Default::default(),
            mempool_error: Default::default(),
        }
    }
}

impl<Ev> From<CheckTx<Ev>> for abci::response::CheckTx
where
    Ev: Into<abci::Event>,
{
    fn from(msg: CheckTx<Ev>) -> Self {
        Self {
            code: msg.code,
            data: msg.data,
            log: msg.log,
            info: msg.info,
            gas_wanted: msg.gas_wanted,
            gas_used: msg.gas_used,
            events: msg.events.into_iter().map(Into::into).collect(),
            codespace: msg.codespace,
            sender: msg.sender,
            priority: msg.priority,
            mempool_error: msg.mempool_error,
        }
    }
}
