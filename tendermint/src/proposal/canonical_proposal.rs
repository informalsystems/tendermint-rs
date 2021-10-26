//! CanonicalProposal

use super::Type;
use crate::block::{Height, Id as BlockId, Round};
use crate::chain::Id as ChainId;
use crate::error::Error;
use crate::prelude::*;
use crate::Time;
use core::convert::{TryFrom, TryInto};
use tendermint_proto::types::CanonicalProposal as RawCanonicalProposal;
use tendermint_proto::Protobuf;

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

impl Protobuf<RawCanonicalProposal> for CanonicalProposal {}

impl TryFrom<RawCanonicalProposal> for CanonicalProposal {
    type Error = Error;

    fn try_from(value: RawCanonicalProposal) -> Result<Self, Self::Error> {
        if value.pol_round < -1 {
            return Err(Error::negative_pol_round());
        }
        let round = Round::try_from(i32::try_from(value.round).map_err(Error::integer_overflow)?)?;
        let pol_round = match value.pol_round {
            -1 => None,
            n => Some(Round::try_from(
                i32::try_from(n).map_err(Error::integer_overflow)?,
            )?),
        };
        // If the Hash is empty in BlockId, the BlockId should be empty.
        // See: https://github.com/informalsystems/tendermint-rs/issues/663
        let block_id = value.block_id.filter(|i| !i.hash.is_empty());
        Ok(CanonicalProposal {
            msg_type: value.r#type.try_into()?,
            height: value.height.try_into()?,
            round,
            pol_round,
            block_id: block_id.map(TryInto::try_into).transpose()?,
            timestamp: value.timestamp.map(|t| t.try_into()).transpose()?,
            chain_id: ChainId::try_from(value.chain_id).unwrap(),
        })
    }
}

impl From<CanonicalProposal> for RawCanonicalProposal {
    fn from(value: CanonicalProposal) -> Self {
        // If the Hash is empty in BlockId, the BlockId should be empty.
        // See: https://github.com/informalsystems/tendermint-rs/issues/663
        let block_id = value.block_id.filter(|i| i != &BlockId::default());
        RawCanonicalProposal {
            r#type: value.msg_type.into(),
            height: value.height.into(),
            round: i32::from(value.round) as i64,
            pol_round: match value.pol_round {
                None => -1,
                Some(p) => i32::from(p) as i64,
            },
            block_id: block_id.map(Into::into),
            timestamp: value.timestamp.map(Into::into),
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::proposal::canonical_proposal::CanonicalProposal;
    use crate::proposal::Type;
    use core::convert::TryFrom;
    use tendermint_proto::types::CanonicalBlockId as RawCanonicalBlockId;
    use tendermint_proto::types::CanonicalPartSetHeader as RawCanonicalPartSetHeader;
    use tendermint_proto::types::CanonicalProposal as RawCanonicalProposal;

    #[test]
    fn canonical_proposal_domain_checks() {
        // RawCanonicalProposal with edge cases to test domain knowledge
        // pol_round = -1 should decode to None
        // block_id with empty hash should decode to None
        let proto_cp = RawCanonicalProposal {
            r#type: 32,
            height: 2,
            round: 4,
            pol_round: -1,
            block_id: Some(RawCanonicalBlockId {
                hash: vec![],
                part_set_header: Some(RawCanonicalPartSetHeader {
                    total: 1,
                    hash: vec![1],
                }),
            }),
            timestamp: None,
            chain_id: "testchain".to_string(),
        };
        let cp = CanonicalProposal::try_from(proto_cp).unwrap();
        assert_eq!(cp.msg_type, Type::Proposal);
        assert!(cp.pol_round.is_none());
        assert!(cp.block_id.is_none());
    }
}
