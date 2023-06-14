use serde::{Deserialize, Serialize};

use crate::{abci::Event, prelude::*};

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

mod v0_34 {
    use super::BeginBlock;
    use tendermint_proto::v0_34 as pb;
    use tendermint_proto::Protobuf;

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

mod v0_37 {
    use super::BeginBlock;
    use tendermint_proto::v0_37 as pb;
    use tendermint_proto::Protobuf;

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
