use crate::prelude::*;

use bytes::Bytes;

use super::super::Event;

#[doc = include_str!("../doc/response-checktx.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct CheckTx {
    /// The response code.
    ///
    /// Transactions where `code != 0` will be rejected; these transactions will
    /// not be broadcast to other nodes or included in a proposal block.
    /// Tendermint attributes no other value to the response code.
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
    /// Events that occurred while checking the transaction.
    pub events: Vec<Event>,
    /// The namespace for the `code`.
    pub codespace: String,
    /// The transaction's sender (e.g. the signer).
    pub sender: String,
    /// The transaction's priority (for mempool ordering).
    pub priority: i64,
    /* mempool_error is contained in the proto, but skipped here:
     * > mempool_error is set by Tendermint.
     * > ABCI applictions creating a ResponseCheckTX should not set mempool_error. */
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::{TryFrom, TryInto};
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<CheckTx> for pb::ResponseCheckTx {
    fn from(check_tx: CheckTx) -> Self {
        Self {
            code: check_tx.code,
            data: check_tx.data,
            log: check_tx.log,
            info: check_tx.info,
            gas_wanted: check_tx.gas_wanted,
            gas_used: check_tx.gas_used,
            events: check_tx.events.into_iter().map(Into::into).collect(),
            codespace: check_tx.codespace,
            sender: check_tx.sender,
            priority: check_tx.priority,
            mempool_error: String::default(),
        }
    }
}

impl TryFrom<pb::ResponseCheckTx> for CheckTx {
    type Error = crate::Error;

    fn try_from(check_tx: pb::ResponseCheckTx) -> Result<Self, Self::Error> {
        Ok(Self {
            code: check_tx.code,
            data: check_tx.data,
            log: check_tx.log,
            info: check_tx.info,
            gas_wanted: check_tx.gas_wanted,
            gas_used: check_tx.gas_used,
            events: check_tx
                .events
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            codespace: check_tx.codespace,
            sender: check_tx.sender,
            priority: check_tx.priority,
        })
    }
}

impl Protobuf<pb::ResponseCheckTx> for CheckTx {}
