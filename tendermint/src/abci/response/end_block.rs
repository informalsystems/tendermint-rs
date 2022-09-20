use super::super::{types::ValidatorUpdate, Event};
use crate::{consensus, prelude::*};

#[doc = include_str!("../doc/response-endblock.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct EndBlock {
    /// Changes to the validator set, if any.
    ///
    /// Setting the voting power to 0 removes a validator.
    pub validator_updates: Vec<ValidatorUpdate>,
    /// Changes to consensus parameters (optional).
    pub consensus_param_updates: Option<consensus::Params>,
    /// Events that occurred while ending the block.
    pub events: Vec<Event>,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

use core::convert::{TryFrom, TryInto};

use tendermint_proto::{abci as pb, Protobuf};

impl From<EndBlock> for pb::ResponseEndBlock {
    fn from(end_block: EndBlock) -> Self {
        Self {
            validator_updates: end_block
                .validator_updates
                .into_iter()
                .map(Into::into)
                .collect(),
            consensus_param_updates: end_block.consensus_param_updates.map(Into::into),
            events: end_block.events.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<pb::ResponseEndBlock> for EndBlock {
    type Error = crate::Error;

    fn try_from(end_block: pb::ResponseEndBlock) -> Result<Self, Self::Error> {
        Ok(Self {
            validator_updates: end_block
                .validator_updates
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            consensus_param_updates: end_block
                .consensus_param_updates
                .map(TryInto::try_into)
                .transpose()?,
            events: end_block
                .events
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl Protobuf<pb::ResponseEndBlock> for EndBlock {}
