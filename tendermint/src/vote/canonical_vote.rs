use crate::chain::Id as ChainId;
use crate::{block, Time};
use crate::{Error, Kind::*};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use tendermint_proto::types::CanonicalVote as RawCanonicalVote;
use tendermint_proto::DomainType;

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
        // The JSON encoding says, if the Hash is empty in BlockId, the BlockId should be empty.
        let block_id =
            if value.block_id.is_some() && value.block_id.clone().unwrap().hash.is_empty() {
                None
            } else {
                value.block_id.map(TryInto::try_into).transpose()?
            };
        Ok(CanonicalVote {
            vote_type: value.r#type.try_into()?,
            height: value.height.try_into()?,
            round: (value.round as i32).try_into()?,
            block_id,
            timestamp: value.timestamp.map(TryInto::try_into).transpose()?,
            chain_id: ChainId::try_from(value.chain_id)?,
        })
    }
}

impl From<CanonicalVote> for RawCanonicalVote {
    fn from(value: CanonicalVote) -> Self {
        // The JSON encoding says, if the Hash is empty in BlockId, the BlockId should be empty.
        // Todo: Does the protobuf encoding have the same rule?
        let block_id =
            if value.block_id.is_some() && value.block_id.clone().unwrap().hash.is_empty() {
                None
            } else {
                value.block_id.map(Into::into)
            };
        RawCanonicalVote {
            r#type: value.vote_type.into(),
            height: value.height.into(),
            round: value.round.value().into(),
            block_id,
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
