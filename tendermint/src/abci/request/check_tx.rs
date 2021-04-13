use crate::prelude::*;

use bytes::Bytes;

#[doc = include_str!("../doc/request-checktx.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CheckTx {
    /// The transaction bytes.
    pub tx: Bytes,
    /// The kind of check to perform.
    ///
    /// Note: this field is called `type` in the protobuf, but we call it `kind`
    /// to avoid the Rust keyword.
    pub kind: CheckTxKind,
}

/// The possible kinds of [`CheckTx`] checks.
///
/// Note: the
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#checktx)
/// calls this `CheckTxType`, but we follow the Rust convention and name it `CheckTxKind`
/// to avoid confusion with Rust types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum CheckTxKind {
    /// A full check is required (the default).
    New = 0,
    /// Indicates that the mempool is initiating a recheck of the transaction.
    Recheck = 1,
}

impl Default for CheckTxKind {
    fn default() -> Self {
        CheckTxKind::New
    }
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::TryFrom;
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<CheckTx> for pb::RequestCheckTx {
    fn from(check_tx: CheckTx) -> Self {
        Self {
            tx: check_tx.tx,
            r#type: check_tx.kind as i32,
        }
    }
}

impl TryFrom<pb::RequestCheckTx> for CheckTx {
    type Error = crate::Error;

    fn try_from(check_tx: pb::RequestCheckTx) -> Result<Self, Self::Error> {
        let kind = match check_tx.r#type {
            0 => CheckTxKind::New,
            1 => CheckTxKind::Recheck,
            _ => Err("unknown checktx type")?,
        };
        Ok(Self {
            tx: check_tx.tx,
            kind,
        })
    }
}

impl Protobuf<pb::RequestCheckTx> for CheckTx {}
