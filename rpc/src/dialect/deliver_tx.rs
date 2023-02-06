use bytes::Bytes;
use serde::{Deserialize, Serialize};

use tendermint::abci::{self, Code};

use crate::prelude::*;
use crate::serializers;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DeliverTx<Ev> {
    /// The response code.
    ///
    /// This code should be `0` only if the transaction is fully valid. However,
    /// invalid transactions included in a block will still be executed against
    /// the application state.
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
    /// Events that occurred while executing the transaction.
    pub events: Vec<Ev>,
    /// The namespace for the `code`.
    pub codespace: String,
}

impl<Ev> Default for DeliverTx<Ev> {
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
        }
    }
}

impl<Ev> From<DeliverTx<Ev>> for abci::response::DeliverTx
where
    Ev: Into<abci::Event>,
{
    fn from(msg: DeliverTx<Ev>) -> Self {
        Self {
            code: msg.code,
            data: msg.data,
            log: msg.log,
            info: msg.info,
            gas_wanted: msg.gas_wanted,
            gas_used: msg.gas_used,
            events: msg.events.into_iter().map(Into::into).collect(),
            codespace: msg.codespace,
        }
    }
}
