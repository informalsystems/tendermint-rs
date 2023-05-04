use crate::prelude::*;
use crate::{
    abci::types::{CommitInfo, Misbehavior},
    account, block, Hash, Time,
};
use bytes::Bytes;

#[doc = include_str!("../doc/request-finalizeblock.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FinalizeBlock {
    /// List of transactions committed as part of the block.
    pub txs: Vec<Bytes>,
    /// Information about the last commit, obtained from the block that was just decided.
    ///
    /// This includes the round, the list of validators, and which validators
    /// signed the last block.
    pub decided_last_commit: CommitInfo,
    /// Evidence of validator misbehavior.
    pub misbehavior: Vec<Misbehavior>,
    /// Merkle root hash of the fields of the decided block.
    pub hash: Hash,
    /// The height of the finalized block.
    pub height: block::Height,
    /// Timestamp of the finalized block.
    pub time: Time,
    /// Merkle root of the next validator set.
    pub next_validators_hash: Hash,
    /// The address of the public key of the original proposer of the block.
    pub proposer_address: account::Id,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

mod v0_38 {
    use super::FinalizeBlock;
    use crate::Error;
    use tendermint_proto::v0_38 as pb;
    use tendermint_proto::Protobuf;

    impl From<FinalizeBlock> for pb::abci::RequestFinalizeBlock {
        fn from(value: FinalizeBlock) -> Self {
            Self {
                txs: value.txs,
                decided_last_commit: Some(value.decided_last_commit.into()),
                misbehavior: value.misbehavior.into_iter().map(Into::into).collect(),
                hash: value.hash.into(),
                height: value.height.into(),
                time: Some(value.time.into()),
                next_validators_hash: value.next_validators_hash.into(),
                proposer_address: value.proposer_address.into(),
            }
        }
    }

    impl TryFrom<pb::abci::RequestFinalizeBlock> for FinalizeBlock {
        type Error = Error;

        fn try_from(message: pb::abci::RequestFinalizeBlock) -> Result<Self, Self::Error> {
            Ok(Self {
                txs: message.txs,
                decided_last_commit: message
                    .decided_last_commit
                    .ok_or_else(Error::missing_last_commit_info)?
                    .try_into()?,
                misbehavior: message
                    .misbehavior
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
                hash: message.hash.try_into()?,
                height: message.height.try_into()?,
                time: message
                    .time
                    .ok_or_else(Error::missing_timestamp)?
                    .try_into()?,
                next_validators_hash: message.next_validators_hash.try_into()?,
                proposer_address: message.proposer_address.try_into()?,
            })
        }
    }

    impl Protobuf<pb::abci::RequestFinalizeBlock> for FinalizeBlock {}
}
