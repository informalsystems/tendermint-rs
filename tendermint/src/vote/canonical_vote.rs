use crate::chain::Id as ChainId;
use crate::{block, Time};
use crate::{Error, Kind::*};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use tendermint_proto::types::CanonicalVote as RawCanonicalVote;
use tendermint_proto::Protobuf;

/// CanonicalVote is used for protobuf encoding a Vote
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(try_from = "RawCanonicalVote", into = "RawCanonicalVote")]
pub struct CanonicalVote {
    /// Type of vote (prevote or precommit)
    pub vote_type: super::Type,

    /// Block height
    pub height: block::Height,

    /// Round
    pub round: block::Round,

    /// Block ID
    //#[serde(deserialize_with = "serializers::parse_non_empty_block_id")] - moved to try_from
    pub block_id: Option<block::Id>,

    /// Timestamp
    pub timestamp: Option<Time>,

    /// Chain ID
    pub chain_id: ChainId,

    /// Unsigned app vote data
    pub unsigned_app_vote_data: String,
}

impl Protobuf<RawCanonicalVote> for CanonicalVote {}

impl TryFrom<RawCanonicalVote> for CanonicalVote {
    type Error = Error;

    fn try_from(value: RawCanonicalVote) -> Result<Self, Self::Error> {
        if value.timestamp.is_none() {
            return Err(NoTimestamp.into());
        }
        if value.round > i32::MAX as i64 {
            // CanonicalVote uses sfixed64, Vote uses int32. They translate to u64 vs i32 in Rust.
            return Err(IntegerOverflow.into());
        }
        // If the Hash is empty in BlockId, the BlockId should be empty.
        // See: https://github.com/informalsystems/tendermint-rs/issues/663
        let block_id = value.block_id.filter(|i| !i.hash.is_empty());
        Ok(CanonicalVote {
            vote_type: value.r#type.try_into()?,
            height: value.height.try_into()?,
            round: (value.round as i32).try_into()?,
            block_id: block_id.map(TryInto::try_into).transpose()?,
            timestamp: value.timestamp.map(TryInto::try_into).transpose()?,
            chain_id: ChainId::try_from(value.chain_id)?,
            unsigned_app_vote_data: value.unsigned_app_vote_data,
        })
    }
}

impl From<CanonicalVote> for RawCanonicalVote {
    fn from(value: CanonicalVote) -> Self {
        // If the Hash is empty in BlockId, the BlockId should be empty.
        // See: https://github.com/informalsystems/tendermint-rs/issues/663
        let block_id = value.block_id.filter(|i| i != &block::Id::default());
        RawCanonicalVote {
            r#type: value.vote_type.into(),
            height: value.height.into(),
            round: value.round.value().into(),
            block_id: block_id.map(Into::into),
            timestamp: value.timestamp.map(Into::into),
            chain_id: value.chain_id.to_string(),
            unsigned_app_vote_data: value.unsigned_app_vote_data,
        }
    }
}

impl CanonicalVote {
    /// Create CanonicalVote from Vote
    pub fn new(vote: super::Vote, chain_id: ChainId) -> CanonicalVote {
        CanonicalVote {
            vote_type: vote.vote_type,
            height: vote.height,
            round: vote.round,
            block_id: vote.block_id,
            timestamp: vote.timestamp,
            chain_id,
            unsigned_app_vote_data: "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::vote::canonical_vote::CanonicalVote;
    use crate::vote::Type;
    use std::convert::TryFrom;
    use tendermint_proto::google::protobuf::Timestamp;
    use tendermint_proto::types::CanonicalBlockId as RawCanonicalBlockId;
    use tendermint_proto::types::CanonicalPartSetHeader as RawCanonicalPartSetHeader;
    use tendermint_proto::types::CanonicalVote as RawCanonicalVote;

    #[test]
    fn canonical_vote_domain_checks() {
        // RawCanonicalVote with edge cases to test domain knowledge
        // block_id with empty hash should decode to None
        // timestamp at EPOCH is still considered valid time
        let proto_cp = RawCanonicalVote {
            r#type: 1,
            height: 2,
            round: 4,
            block_id: Some(RawCanonicalBlockId {
                hash: vec![],
                part_set_header: Some(RawCanonicalPartSetHeader {
                    total: 1,
                    hash: vec![1],
                }),
            }),
            timestamp: Some(Timestamp {
                seconds: 0,
                nanos: 0,
            }),
            chain_id: "testchain".to_string(),
            unsigned_app_vote_data: "".to_string(),
        };
        let cp = CanonicalVote::try_from(proto_cp).unwrap();
        assert_eq!(cp.vote_type, Type::Prevote);
        assert!(cp.block_id.is_none());
        assert!(cp.timestamp.is_some());

        // No timestamp is not acceptable
        // See: https://github.com/informalsystems/tendermint-rs/issues/649
        let mut proto_cp: RawCanonicalVote = cp.into();
        proto_cp.timestamp = None;
        assert!(CanonicalVote::try_from(proto_cp).is_err());
    }
}
