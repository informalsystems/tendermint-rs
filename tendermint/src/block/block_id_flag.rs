/// Indicates whether the validator voted for a block, nil, or did not vote at all
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BlockIdFlag {
    /// The vote was not received.
    Absent,
    /// Voted for a block.
    Commit,
    /// Voted for nil.
    Nil,
}

tendermint_pb_modules! {
    use super::BlockIdFlag;
    use crate::{error::Error, prelude::*};
    use pb::types::BlockIdFlag as RawBlockIdFlag;

    impl TryFrom<RawBlockIdFlag> for BlockIdFlag {
        type Error = Error;

        fn try_from(value: RawBlockIdFlag) -> Result<Self, Self::Error> {
            match value {
                RawBlockIdFlag::Absent => Ok(BlockIdFlag::Absent),
                RawBlockIdFlag::Commit => Ok(BlockIdFlag::Commit),
                RawBlockIdFlag::Nil => Ok(BlockIdFlag::Nil),
                _ => Err(Error::block_id_flag()),            }
        }
    }

    impl From<BlockIdFlag> for RawBlockIdFlag {
        fn from(value: BlockIdFlag) -> RawBlockIdFlag {
            match value {
                BlockIdFlag::Absent => RawBlockIdFlag::Absent,
                BlockIdFlag::Commit => RawBlockIdFlag::Commit,
                BlockIdFlag::Nil => RawBlockIdFlag::Nil,
            }
        }
    }
}
