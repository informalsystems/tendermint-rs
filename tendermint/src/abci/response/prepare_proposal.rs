use bytes::Bytes;

use crate::prelude::*;

#[doc = include_str!("../doc/response-prepareproposal.md")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrepareProposal {
    pub txs: Vec<Bytes>,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

mod v0_37 {
    use super::PrepareProposal;
    use tendermint_proto::v0_37::abci as pb;
    use tendermint_proto::Protobuf;

    impl From<PrepareProposal> for pb::ResponsePrepareProposal {
        fn from(value: PrepareProposal) -> Self {
            Self { txs: value.txs }
        }
    }

    impl TryFrom<pb::ResponsePrepareProposal> for PrepareProposal {
        type Error = crate::Error;

        fn try_from(message: pb::ResponsePrepareProposal) -> Result<Self, Self::Error> {
            Ok(Self { txs: message.txs })
        }
    }

    impl Protobuf<pb::ResponsePrepareProposal> for PrepareProposal {}
}

mod v0_38 {
    use super::PrepareProposal;
    use tendermint_proto::v0_38::abci as pb;
    use tendermint_proto::Protobuf;

    impl From<PrepareProposal> for pb::ResponsePrepareProposal {
        fn from(value: PrepareProposal) -> Self {
            Self { txs: value.txs }
        }
    }

    impl TryFrom<pb::ResponsePrepareProposal> for PrepareProposal {
        type Error = crate::Error;

        fn try_from(message: pb::ResponsePrepareProposal) -> Result<Self, Self::Error> {
            Ok(Self { txs: message.txs })
        }
    }

    impl Protobuf<pb::ResponsePrepareProposal> for PrepareProposal {}
}
