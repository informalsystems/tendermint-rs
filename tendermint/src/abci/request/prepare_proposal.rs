use crate::prelude::*;
use crate::{
    abci::types::{ExtendedCommitInfo, Misbehavior},
    account, block, Hash, Time,
};

use bytes::Bytes;

#[doc = include_str!("../doc/request-prepareproposal.md")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrepareProposal {
    /// the modified transactions cannot exceed this size.
    pub max_tx_bytes: i64,
    /// txs is an array of transactions that will be included in a block,
    /// sent to the app for possible modifications.
    pub txs: Vec<Bytes>,
    pub local_last_commit: Option<ExtendedCommitInfo>,
    pub misbehavior: Vec<Misbehavior>,
    pub height: block::Height,
    pub time: Time,
    pub next_validators_hash: Hash,
    /// address of the public key of the validator proposing the block.
    pub proposer_address: account::Id,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

mod v0_37 {
    use super::PrepareProposal;
    use crate::{prelude::*, Error};
    use tendermint_proto::v0_37::abci as pb;
    use tendermint_proto::Protobuf;

    impl From<PrepareProposal> for pb::RequestPrepareProposal {
        fn from(value: PrepareProposal) -> Self {
            Self {
                max_tx_bytes: value.max_tx_bytes,
                txs: value.txs,
                local_last_commit: value.local_last_commit.map(Into::into),
                misbehavior: value.misbehavior.into_iter().map(Into::into).collect(),
                height: value.height.into(),
                time: Some(value.time.into()),
                next_validators_hash: value.next_validators_hash.into(),
                proposer_address: value.proposer_address.into(),
            }
        }
    }

    impl TryFrom<pb::RequestPrepareProposal> for PrepareProposal {
        type Error = Error;

        fn try_from(message: pb::RequestPrepareProposal) -> Result<Self, Self::Error> {
            let req = Self {
                max_tx_bytes: message.max_tx_bytes,
                txs: message.txs,
                local_last_commit: message
                    .local_last_commit
                    .map(TryInto::try_into)
                    .transpose()?,
                misbehavior: message
                    .misbehavior
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?,
                height: message.height.try_into()?,
                time: message
                    .time
                    .ok_or_else(Error::missing_timestamp)?
                    .try_into()?,
                next_validators_hash: message.next_validators_hash.try_into()?,
                proposer_address: message.proposer_address.try_into()?,
            };
            Ok(req)
        }
    }

    impl Protobuf<pb::RequestPrepareProposal> for PrepareProposal {}
}

mod v0_38 {
    use super::PrepareProposal;
    use crate::{prelude::*, Error};
    use tendermint_proto::v0_38::abci as pb;
    use tendermint_proto::Protobuf;

    impl From<PrepareProposal> for pb::RequestPrepareProposal {
        fn from(value: PrepareProposal) -> Self {
            Self {
                max_tx_bytes: value.max_tx_bytes,
                txs: value.txs,
                local_last_commit: value.local_last_commit.map(Into::into),
                misbehavior: value.misbehavior.into_iter().map(Into::into).collect(),
                height: value.height.into(),
                time: Some(value.time.into()),
                next_validators_hash: value.next_validators_hash.into(),
                proposer_address: value.proposer_address.into(),
            }
        }
    }

    impl TryFrom<pb::RequestPrepareProposal> for PrepareProposal {
        type Error = Error;

        fn try_from(message: pb::RequestPrepareProposal) -> Result<Self, Self::Error> {
            let req = Self {
                max_tx_bytes: message.max_tx_bytes,
                txs: message.txs,
                local_last_commit: message
                    .local_last_commit
                    .map(TryInto::try_into)
                    .transpose()?,
                misbehavior: message
                    .misbehavior
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?,
                height: message.height.try_into()?,
                time: message
                    .time
                    .ok_or_else(Error::missing_timestamp)?
                    .try_into()?,
                next_validators_hash: message.next_validators_hash.try_into()?,
                proposer_address: message.proposer_address.try_into()?,
            };
            Ok(req)
        }
    }

    impl Protobuf<pb::RequestPrepareProposal> for PrepareProposal {}
}
