use crate::prelude::*;

#[doc = include_str!("../doc/request-loadsnapshotchunk.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LoadSnapshotChunk {
    /// The height of the snapshot the chunks belong to.
    pub height: u64,
    /// An application-specific identifier of the format of the snapshot chunk.
    pub format: u32,
    /// The chunk index, starting from `0` for the initial chunk.
    pub chunk: u32,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::TryFrom;
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<LoadSnapshotChunk> for pb::RequestLoadSnapshotChunk {
    fn from(load_snapshot_chunk: LoadSnapshotChunk) -> Self {
        Self {
            height: load_snapshot_chunk.height,
            format: load_snapshot_chunk.format,
            chunk: load_snapshot_chunk.chunk,
        }
    }
}

impl TryFrom<pb::RequestLoadSnapshotChunk> for LoadSnapshotChunk {
    type Error = crate::Error;

    fn try_from(load_snapshot_chunk: pb::RequestLoadSnapshotChunk) -> Result<Self, Self::Error> {
        Ok(Self {
            height: load_snapshot_chunk.height,
            format: load_snapshot_chunk.format,
            chunk: load_snapshot_chunk.chunk,
        })
    }
}

impl Protobuf<pb::RequestLoadSnapshotChunk> for LoadSnapshotChunk {}
