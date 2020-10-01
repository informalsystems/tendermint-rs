//! CanonicalProposal

use super::Type;
use crate::block::{Height, Id as BlockId, Round};
use crate::chain::Id as ChainId;
use crate::Time;
use crate::{Error, Kind};
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use tendermint_proto::types::CanonicalProposal as RawCanonicalProposal;
use tendermint_proto::DomainType;

/// CanonicalProposal for signing
#[derive(Clone, PartialEq)]
pub struct CanonicalProposal {
    /// type alias for byte
    pub msg_type: Type,
    /// canonicalization requires fixed size encoding here
    pub height: Height,
    /// canonicalization requires fixed size encoding here
    pub round: Round,
    /// POL round
    pub pol_round: Option<Round>,
    /// Block ID
    pub block_id: Option<BlockId>,
    /// Timestamp
    pub timestamp: Option<Time>,
    /// Chain ID
    pub chain_id: ChainId,
}

impl DomainType<RawCanonicalProposal> for CanonicalProposal {}

impl TryFrom<RawCanonicalProposal> for CanonicalProposal {
    type Error = Error;

    fn try_from(value: RawCanonicalProposal) -> Result<Self, Self::Error> {
        if value.pol_round < -1 {
            return Err(Kind::NegativePOLRound.into());
        }
        let round = Round::try_from(
            i32::try_from(value.round).map_err(|e| Kind::IntegerOverflow.context(e))?,
        )?;
        let pol_round = match value.pol_round {
            -1 => None,
            n => Some(Round::try_from(
                i32::try_from(n).map_err(|e| Kind::IntegerOverflow.context(e))?,
            )?),
        };
        Ok(CanonicalProposal {
            msg_type: value.r#type.try_into()?,
            height: value.height.try_into()?,
            round,
            pol_round,
            block_id: match value.block_id {
                None => None,
                Some(block_id) => Some(block_id.try_into()?),
            },
            timestamp: match value.timestamp {
                None => None,
                Some(t) => Some(t.try_into()?),
            },
            chain_id: ChainId::from_str(value.chain_id.as_str()).unwrap(),
        })
    }
}

impl From<CanonicalProposal> for RawCanonicalProposal {
    fn from(value: CanonicalProposal) -> Self {
        RawCanonicalProposal {
            r#type: value.msg_type.into(),
            height: value.height.into(),
            round: i32::from(value.round) as i64,
            pol_round: match value.pol_round {
                None => -1,
                Some(p) => i32::from(p) as i64,
            },
            block_id: match value.block_id {
                None => None,
                Some(block_id) => Some(block_id.into()),
            },
            timestamp: value.timestamp.map(|t| t.into()),
            chain_id: value.chain_id.as_str().to_string(),
        }
    }
}

impl CanonicalProposal {
    /// Create CanonicalProposal from Proposal
    pub fn new(proposal: super::Proposal, chain_id: ChainId) -> CanonicalProposal {
        CanonicalProposal {
            msg_type: proposal.msg_type,
            height: proposal.height,
            round: proposal.round,
            pol_round: proposal.pol_round,
            block_id: proposal.block_id,
            timestamp: proposal.timestamp,
            chain_id,
        }
    }
}
