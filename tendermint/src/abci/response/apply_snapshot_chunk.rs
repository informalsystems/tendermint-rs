use crate::prelude::*;

#[doc = include_str!("../doc/response-applysnapshotchunk.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ApplySnapshotChunk {
    /// The result of applying the snapshot chunk.
    pub result: ApplySnapshotChunkResult,
    /// Refetch and reapply the given chunks, regardless of `result`.
    ///
    /// Only the listed chunks will be refetched, and reapplied in sequential
    /// order.
    pub refetch_chunks: Vec<u32>,
    /// Reject the given P2P senders, regardless of `result`.
    ///
    /// Any chunks already applied will not be refetched unless explicitly
    /// requested, but queued chunks from these senders will be discarded, and
    /// new chunks or other snapshots rejected.
    pub reject_senders: Vec<String>,
}

/// The result of applying a snapshot chunk.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ApplySnapshotChunkResult {
    /// Unknown result, abort all snapshot restoration.
    Unknown = 0,
    /// The chunk was accepted.
    Accept = 1,
    /// Abort snapshot restoration, and don't try any other snapshots.
    Abort = 2,
    /// Reapply this chunk, combine with
    /// [`refetch_chunks`](ApplySnapshotChunk::refetch_chunks) and
    /// [`reject_senders`](ApplySnapshotChunk::reject_senders) as appropriate.
    Retry = 3,
    /// Restart this snapshot from
    /// [`OfferSnapshot`](super::super::request::OfferSnapshot),
    /// reusing chunks unless instructed otherwise.
    RetrySnapshot = 4,
    /// Reject this snapshot, try a different one.
    RejectSnapshot = 5,
}

impl Default for ApplySnapshotChunkResult {
    fn default() -> Self {
        Self::Unknown
    }
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::{ApplySnapshotChunk, ApplySnapshotChunkResult};

    impl From<ApplySnapshotChunk> for pb::abci::ResponseApplySnapshotChunk {
        fn from(apply_snapshot_chunk: ApplySnapshotChunk) -> Self {
            Self {
                result: apply_snapshot_chunk.result as i32,
                refetch_chunks: apply_snapshot_chunk.refetch_chunks,
                reject_senders: apply_snapshot_chunk.reject_senders,
            }
        }
    }

    impl TryFrom<pb::abci::ResponseApplySnapshotChunk> for ApplySnapshotChunk {
        type Error = crate::Error;

        fn try_from(apply_snapshot_chunk: pb::abci::ResponseApplySnapshotChunk) -> Result<Self, Self::Error> {
            let result = match apply_snapshot_chunk.result {
                0 => ApplySnapshotChunkResult::Unknown,
                1 => ApplySnapshotChunkResult::Accept,
                2 => ApplySnapshotChunkResult::Abort,
                3 => ApplySnapshotChunkResult::Retry,
                4 => ApplySnapshotChunkResult::RetrySnapshot,
                5 => ApplySnapshotChunkResult::RejectSnapshot,
                _ => return Err(crate::Error::unsupported_apply_snapshot_chunk_result()),
            };
            Ok(Self {
                result,
                refetch_chunks: apply_snapshot_chunk.refetch_chunks,
                reject_senders: apply_snapshot_chunk.reject_senders,
            })
        }
    }

    impl Protobuf<pb::abci::ResponseApplySnapshotChunk> for ApplySnapshotChunk {}
}
