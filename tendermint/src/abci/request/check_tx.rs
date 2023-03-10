use bytes::Bytes;

use crate::prelude::*;

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
#[derive(Default)]
pub enum CheckTxKind {
    /// A full check is required (the default).
    #[default]
    New = 0,
    /// Indicates that the mempool is initiating a recheck of the transaction.
    Recheck = 1,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::{CheckTx, CheckTxKind};

    impl From<CheckTx> for pb::abci::RequestCheckTx {
        fn from(check_tx: CheckTx) -> Self {
            Self {
                tx: check_tx.tx,
                r#type: check_tx.kind as i32,
            }
        }
    }

    impl TryFrom<pb::abci::RequestCheckTx> for CheckTx {
        type Error = crate::Error;

        fn try_from(check_tx: pb::abci::RequestCheckTx) -> Result<Self, Self::Error> {
            let kind = match check_tx.r#type {
                0 => CheckTxKind::New,
                1 => CheckTxKind::Recheck,
                _ => return Err(crate::Error::unsupported_check_tx_type()),
            };
            Ok(Self {
                tx: check_tx.tx,
                kind,
            })
        }
    }

    impl Protobuf<pb::abci::RequestCheckTx> for CheckTx {}
}
