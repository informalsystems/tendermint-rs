use crate::prelude::*;

use bytes::Bytes;

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

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::TryFrom;
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<LoadSnapshotChunk> for pb::ResponseLoadSnapshotChunk {
    fn from(load_snapshot_chunk: LoadSnapshotChunk) -> Self {
        Self {
            chunk: load_snapshot_chunk.chunk,
        }
    }
}

impl TryFrom<pb::ResponseLoadSnapshotChunk> for LoadSnapshotChunk {
    type Error = crate::Error;

    fn try_from(load_snapshot_chunk: pb::ResponseLoadSnapshotChunk) -> Result<Self, Self::Error> {
        Ok(Self {
            chunk: load_snapshot_chunk.chunk,
        })
    }
}

impl Protobuf<pb::ResponseLoadSnapshotChunk> for LoadSnapshotChunk {}
