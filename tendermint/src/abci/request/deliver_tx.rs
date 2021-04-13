use crate::prelude::*;

use bytes::Bytes;

#[doc = include_str!("../doc/request-delivertx.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DeliverTx {
    /// The bytes of the transaction to execute.
    pub tx: Bytes,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::TryFrom;
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<DeliverTx> for pb::RequestDeliverTx {
    fn from(deliver_tx: DeliverTx) -> Self {
        Self { tx: deliver_tx.tx }
    }
}

impl TryFrom<pb::RequestDeliverTx> for DeliverTx {
    type Error = crate::Error;

    fn try_from(deliver_tx: pb::RequestDeliverTx) -> Result<Self, Self::Error> {
        Ok(Self { tx: deliver_tx.tx })
    }
}

impl Protobuf<pb::RequestDeliverTx> for DeliverTx {}
