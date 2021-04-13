use crate::prelude::*;

use bytes::Bytes;

use super::super::Event;

#[doc = include_str!("../doc/response-delivertx.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct DeliverTx {
    /// The response code.
    ///
    /// This code should be `0` only if the transaction is fully valid. However,
    /// invalid transactions included in a block will still be executed against
    /// the application state.
    pub code: u32,
    /// Result bytes, if any.
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
    pub gas_wanted: i64,
    /// Amount of gas consumed by the transaction.
    pub gas_used: i64,
    /// Events that occurred while executing the transaction.
    pub events: Vec<Event>,
    /// The namespace for the `code`.
    pub codespace: String,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::{TryFrom, TryInto};
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<DeliverTx> for pb::ResponseDeliverTx {
    fn from(deliver_tx: DeliverTx) -> Self {
        Self {
            code: deliver_tx.code,
            data: deliver_tx.data,
            log: deliver_tx.log,
            info: deliver_tx.info,
            gas_wanted: deliver_tx.gas_wanted,
            gas_used: deliver_tx.gas_used,
            events: deliver_tx.events.into_iter().map(Into::into).collect(),
            codespace: deliver_tx.codespace,
        }
    }
}

impl TryFrom<pb::ResponseDeliverTx> for DeliverTx {
    type Error = crate::Error;

    fn try_from(deliver_tx: pb::ResponseDeliverTx) -> Result<Self, Self::Error> {
        Ok(Self {
            code: deliver_tx.code,
            data: deliver_tx.data,
            log: deliver_tx.log,
            info: deliver_tx.info,
            gas_wanted: deliver_tx.gas_wanted,
            gas_used: deliver_tx.gas_used,
            events: deliver_tx
                .events
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            codespace: deliver_tx.codespace,
        })
    }
}

impl Protobuf<pb::ResponseDeliverTx> for DeliverTx {}
