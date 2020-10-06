use crate::chain;
use crate::chain::Id as ChainId;
use crate::{block, Time};
use crate::{Error, Kind::*};
use std::convert::{TryFrom, TryInto};
use tendermint_proto::types::CanonicalVote as RawCanonicalVote;
use tendermint_proto::DomainType;
use {
    crate::serializers,
    serde::{Deserialize, Serialize},
};

/// CanonicalVote is used for protobuf encoding a Vote
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CanonicalVote {
    /// Type of vote (prevote or precommit)
    #[serde(rename = "type")]
    pub vote_type: super::Type,

    /// Block height
    pub height: block::Height,

    /// Round
    #[serde(with = "serializers::from_str")]
    pub round: block::Round,

    /// Block ID
    #[serde(deserialize_with = "serializers::parse_non_empty_block_id")]
    pub block_id: Option<block::Id>,

    /// Timestamp
    pub timestamp: Option<Time>,

    /// Chain ID
    pub chain_id: chain::Id,
}

impl DomainType<RawCanonicalVote> for CanonicalVote {}

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
        Ok(CanonicalVote {
            vote_type: value.r#type.try_into()?,
            height: value.height.try_into()?,
            round: (value.round as i32).try_into()?,
            block_id: value.block_id.map(TryInto::try_into).transpose()?,
            timestamp: value.timestamp.map(TryInto::try_into).transpose()?,
            chain_id: chain::Id::try_from(value.chain_id)?,
        })
    }
}

impl From<CanonicalVote> for RawCanonicalVote {
    fn from(value: CanonicalVote) -> Self {
        RawCanonicalVote {
            r#type: value.vote_type.into(),
            height: value.height.into(),
            round: value.round.value().into(),
            block_id: value.block_id.map(Into::into),
            timestamp: value.timestamp.map(Into::into),
            chain_id: value.chain_id.to_string(),
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
        }
    }
}
