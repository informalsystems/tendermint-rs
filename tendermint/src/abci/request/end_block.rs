use crate::prelude::*;

#[doc = include_str!("../doc/request-endblock.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EndBlock {
    /// The height of the block just executed.
    pub height: i64,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::EndBlock;

    impl From<EndBlock> for pb::abci::RequestEndBlock {
        fn from(end_block: EndBlock) -> Self {
            Self {
                height: end_block.height,
            }
        }
    }

    impl TryFrom<pb::abci::RequestEndBlock> for EndBlock {
        type Error = crate::Error;

        fn try_from(end_block: pb::abci::RequestEndBlock) -> Result<Self, Self::Error> {
            Ok(Self {
                height: end_block.height,
            })
        }
    }

    impl Protobuf<pb::abci::RequestEndBlock> for EndBlock {}
}
