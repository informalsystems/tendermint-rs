use serde::{Deserialize, Serialize};

use super::super::Event;
use crate::prelude::*;

#[doc = include_str!("../doc/response-beginblock.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct BeginBlock {
    /// Events that occurred while beginning the block.
    #[serde(default)]
    pub events: Vec<Event>,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

use core::convert::{TryFrom, TryInto};

use tendermint_proto::{abci as pb, Protobuf};

impl From<BeginBlock> for pb::ResponseBeginBlock {
    fn from(begin_block: BeginBlock) -> Self {
        Self {
            events: begin_block.events.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<pb::ResponseBeginBlock> for BeginBlock {
    type Error = crate::Error;

    fn try_from(begin_block: pb::ResponseBeginBlock) -> Result<Self, Self::Error> {
        Ok(Self {
            events: begin_block
                .events
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl Protobuf<pb::ResponseBeginBlock> for BeginBlock {}
