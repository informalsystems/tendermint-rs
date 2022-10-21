use bytes::Bytes;

// bring into scope for doc links
#[allow(unused)]
use super::{super::types::Snapshot, Info, LoadSnapshotChunk};
use crate::prelude::*;

/// Applies a snapshot chunk.
///
/// The application can choose to refetch chunks and/or ban P2P peers as
/// appropriate. Tendermint will not do this unless instructed by the
/// application.
///
/// The application may want to verify each chunk, e.g., by attaching chunk
/// hashes in [`Snapshot::metadata`] and/or incrementally verifying contents
/// against `app_hash`.
///
/// When all chunks have been accepted, Tendermint will make an ABCI [`Info`]
/// request to verify that `last_block_app_hash` and `last_block_height` match
/// the expected values, and record the `app_version` in the node state. It then
/// switches to fast sync or consensus and joins the network.
///
/// If Tendermint is unable to retrieve the next chunk after some time (e.g.,
/// because no suitable peers are available), it will reject the snapshot and try
/// a different one via `OfferSnapshot`. The application should be prepared to
/// reset and accept it or abort as appropriate.
///
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#applysnapshotchunk)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ApplySnapshotChunk {
    /// The chunk index, starting from `0`.  Tendermint applies chunks sequentially.
    pub index: u32,
    /// The binary chunk contents, as returned by [`LoadSnapshotChunk`].
    pub chunk: Bytes,
    /// The P2P ID of the node who sent this chunk.
    pub sender: String,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::ApplySnapshotChunk;

    impl From<ApplySnapshotChunk> for pb::abci::RequestApplySnapshotChunk {
        fn from(apply_snapshot_chunk: ApplySnapshotChunk) -> Self {
            Self {
                index: apply_snapshot_chunk.index,
                chunk: apply_snapshot_chunk.chunk,
                sender: apply_snapshot_chunk.sender,
            }
        }
    }

    impl TryFrom<pb::abci::RequestApplySnapshotChunk> for ApplySnapshotChunk {
        type Error = crate::Error;

        fn try_from(apply_snapshot_chunk: pb::abci::RequestApplySnapshotChunk) -> Result<Self, Self::Error> {
            Ok(Self {
                index: apply_snapshot_chunk.index,
                chunk: apply_snapshot_chunk.chunk,
                sender: apply_snapshot_chunk.sender,
            })
        }
    }

    impl Protobuf<pb::abci::RequestApplySnapshotChunk> for ApplySnapshotChunk {}
}
