use bytes::Bytes;

use crate::abci::types::{CommitInfo, Misbehavior};
use crate::prelude::*;
use crate::{account, block, Hash, Time};

#[doc = include_str!("../doc/request-extendvote.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExtendVote {
    pub hash: Hash,
    pub height: block::Height,
    pub time: Time,
    pub txs: Vec<Bytes>,
    pub proposed_last_commit: Option<CommitInfo>,
    pub misbehavior: Vec<Misbehavior>,
    pub next_validators_hash: Hash,
    pub proposer_address: account::Id,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

mod v0_38 {
    use super::ExtendVote;
    use crate::{prelude::*, Error};
    use tendermint_proto::v0_38 as pb;
    use tendermint_proto::Protobuf;

    impl From<ExtendVote> for pb::abci::RequestExtendVote {
        fn from(extend_vote: ExtendVote) -> Self {
            Self {
                hash: extend_vote.hash.into(),
                height: extend_vote.height.into(),
                time: Some(extend_vote.time.into()),
                txs: extend_vote.txs,
                proposed_last_commit: extend_vote.proposed_last_commit.map(Into::into),
                misbehavior: extend_vote
                    .misbehavior
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                next_validators_hash: extend_vote.next_validators_hash.into(),
                proposer_address: extend_vote.proposer_address.into(),
            }
        }
    }

    impl TryFrom<pb::abci::RequestExtendVote> for ExtendVote {
        type Error = Error;

        fn try_from(message: pb::abci::RequestExtendVote) -> Result<Self, Self::Error> {
            Ok(Self {
                hash: message.hash.try_into()?,
                height: message.height.try_into()?,
                time: message
                    .time
                    .ok_or_else(Error::missing_timestamp)?
                    .try_into()?,
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
                next_validators_hash: message.next_validators_hash.try_into()?,
                proposer_address: message.proposer_address.try_into()?,
            })
        }
    }

    impl Protobuf<pb::abci::RequestExtendVote> for ExtendVote {}
}
