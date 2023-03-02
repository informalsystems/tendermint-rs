use bytes::Bytes;

use crate::prelude::*;

#[doc = include_str!("../doc/response-loadsnapshotchunk.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct LoadSnapshotChunk {
    /// The binary chunk contents, in an arbitrary format.
    ///
    /// Chunk messages cannot be larger than 16MB *including metadata*, so 10MB
    /// is a good starting point.
    pub chunk: Bytes,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::LoadSnapshotChunk;

    impl From<LoadSnapshotChunk> for pb::abci::ResponseLoadSnapshotChunk {
        fn from(load_snapshot_chunk: LoadSnapshotChunk) -> Self {
            Self {
                chunk: load_snapshot_chunk.chunk,
            }
        }
    }

    impl TryFrom<pb::abci::ResponseLoadSnapshotChunk> for LoadSnapshotChunk {
        type Error = crate::Error;

        fn try_from(load_snapshot_chunk: pb::abci::ResponseLoadSnapshotChunk) -> Result<Self, Self::Error> {
            Ok(Self {
                chunk: load_snapshot_chunk.chunk,
            })
        }
    }

    impl Protobuf<pb::abci::ResponseLoadSnapshotChunk> for LoadSnapshotChunk {}
}
