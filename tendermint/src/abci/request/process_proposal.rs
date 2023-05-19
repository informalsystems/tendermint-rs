use crate::prelude::*;
use crate::{
    abci::types::{CommitInfo, Misbehavior},
    account, block, Hash, Time,
};

use bytes::Bytes;

#[doc = include_str!("../doc/request-processproposal.md")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessProposal {
    /// txs is an array of transactions that will be included in a block,
    /// sent to the app for possible modifications.
    pub txs: Vec<Bytes>,
    pub proposed_last_commit: Option<CommitInfo>,
    pub misbehavior: Vec<Misbehavior>,
    pub hash: Hash,
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
    use super::ProcessProposal;
    use crate::{prelude::*, Error};
    use tendermint_proto::v0_37::abci as pb;
    use tendermint_proto::Protobuf;

    impl From<ProcessProposal> for pb::RequestProcessProposal {
        fn from(value: ProcessProposal) -> Self {
            Self {
                txs: value.txs,
                proposed_last_commit: value.proposed_last_commit.map(Into::into),
                misbehavior: value.misbehavior.into_iter().map(Into::into).collect(),
                hash: value.hash.into(),
                height: value.height.into(),
                time: Some(value.time.into()),
                next_validators_hash: value.next_validators_hash.into(),
                proposer_address: value.proposer_address.into(),
            }
        }
    }

    impl TryFrom<pb::RequestProcessProposal> for ProcessProposal {
        type Error = Error;

        fn try_from(message: pb::RequestProcessProposal) -> Result<Self, Self::Error> {
            let req = Self {
                txs: message.txs,
                proposed_last_commit: message
                    .proposed_last_commit
                    .map(TryInto::try_into)
                    .transpose()?,
                misbehavior: message
                    .misbehavior
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?,
                hash: message.hash.try_into()?,
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

    impl Protobuf<pb::RequestProcessProposal> for ProcessProposal {}
}

mod v0_38 {
    use super::ProcessProposal;
    use crate::{prelude::*, Error};
    use tendermint_proto::v0_38::abci as pb;
    use tendermint_proto::Protobuf;

    impl From<ProcessProposal> for pb::RequestProcessProposal {
        fn from(value: ProcessProposal) -> Self {
            Self {
                txs: value.txs,
                proposed_last_commit: value.proposed_last_commit.map(Into::into),
                misbehavior: value.misbehavior.into_iter().map(Into::into).collect(),
                hash: value.hash.into(),
                height: value.height.into(),
                time: Some(value.time.into()),
                next_validators_hash: value.next_validators_hash.into(),
                proposer_address: value.proposer_address.into(),
            }
        }
    }

    impl TryFrom<pb::RequestProcessProposal> for ProcessProposal {
        type Error = Error;

        fn try_from(message: pb::RequestProcessProposal) -> Result<Self, Self::Error> {
            let req = Self {
                txs: message.txs,
                proposed_last_commit: message
                    .proposed_last_commit
                    .map(TryInto::try_into)
                    .transpose()?,
                misbehavior: message
                    .misbehavior
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?,
                hash: message.hash.try_into()?,
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

    impl Protobuf<pb::RequestProcessProposal> for ProcessProposal {}
}
