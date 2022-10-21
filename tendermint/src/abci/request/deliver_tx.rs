use bytes::Bytes;

use crate::prelude::*;

#[doc = include_str!("../doc/request-delivertx.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DeliverTx {
    /// The bytes of the transaction to execute.
    pub tx: Bytes,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::DeliverTx;

    impl From<DeliverTx> for pb::abci::RequestDeliverTx {
        fn from(deliver_tx: DeliverTx) -> Self {
            Self { tx: deliver_tx.tx }
        }
    }

    impl TryFrom<pb::abci::RequestDeliverTx> for DeliverTx {
        type Error = crate::Error;

        fn try_from(deliver_tx: pb::abci::RequestDeliverTx) -> Result<Self, Self::Error> {
            Ok(Self { tx: deliver_tx.tx })
        }
    }

    impl Protobuf<pb::abci::RequestDeliverTx> for DeliverTx {}
}
