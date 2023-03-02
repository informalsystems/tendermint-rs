use super::super::Event;
use crate::prelude::*;

#[doc = include_str!("../doc/response-beginblock.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct BeginBlock {
    /// Events that occurred while beginning the block.
    pub events: Vec<Event>,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::BeginBlock;

    impl From<BeginBlock> for pb::abci::ResponseBeginBlock {
        fn from(begin_block: BeginBlock) -> Self {
            Self {
                events: begin_block.events.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl TryFrom<pb::abci::ResponseBeginBlock> for BeginBlock {
        type Error = crate::Error;

        fn try_from(begin_block: pb::abci::ResponseBeginBlock) -> Result<Self, Self::Error> {
            Ok(Self {
                events: begin_block
                    .events
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
            })
        }
    }

    impl Protobuf<pb::abci::ResponseBeginBlock> for BeginBlock {}
}
