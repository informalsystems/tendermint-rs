//! ABCI responses and response data.

// IMPORTANT NOTE ON DOCUMENTATION:
//
// The documentation for each request type is adapted from the ABCI Methods and
// Types spec document. However, the same logical request may appear three
// times, as a struct with the request data, as a Request variant, and as a
// CategoryRequest variant.
//
// To avoid duplication, this documentation is stored in the doc/ folder in
// individual .md files, which are pasted onto the relevant items using #[doc =
// include_str!(...)].
//
// This is also why certain submodules have #[allow(unused)] imports to bring
// items into scope for doc links, rather than changing the doc links -- it
// allows the doc comments to be copied without editing.

// bring into scope for doc links
#[allow(unused)]
use super::types::Snapshot;
use crate::prelude::*;

mod apply_snapshot_chunk;
mod begin_block;
mod check_tx;
mod commit;
mod deliver_tx;
mod echo;
mod end_block;
mod exception;
mod info;
mod init_chain;
mod list_snapshots;
mod load_snapshot_chunk;
mod offer_snapshot;
mod prepare_proposal;
mod process_proposal;
mod query;
mod set_option;

pub use apply_snapshot_chunk::{ApplySnapshotChunk, ApplySnapshotChunkResult};
pub use begin_block::BeginBlock;
pub use check_tx::CheckTx;
pub use commit::Commit;
pub use deliver_tx::DeliverTx;
pub use echo::Echo;
pub use end_block::EndBlock;
pub use exception::Exception;
pub use info::Info;
pub use init_chain::InitChain;
pub use list_snapshots::ListSnapshots;
pub use load_snapshot_chunk::LoadSnapshotChunk;
pub use offer_snapshot::OfferSnapshot;
pub use prepare_proposal::PrepareProposal;
pub use process_proposal::ProcessProposal;
pub use query::Query;
pub use set_option::SetOption;

/// The consensus category of ABCI responses.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ConsensusResponse {
    #[doc = include_str!("doc/response-initchain.md")]
    InitChain(InitChain),
    #[doc = include_str!("doc/response-prepareproposal.md")]
    PrepareProposal(PrepareProposal),
    #[doc = include_str!("doc/response-processproposal.md")]
    ProcessProposal(ProcessProposal),
    #[doc = include_str!("doc/response-beginblock.md")]
    BeginBlock(BeginBlock),
    #[doc = include_str!("doc/response-delivertx.md")]
    DeliverTx(DeliverTx),
    #[doc = include_str!("doc/response-endblock.md")]
    EndBlock(EndBlock),
    #[doc = include_str!("doc/response-commit.md")]
    Commit(Commit),
}

/// The mempool category of ABCI responses.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MempoolResponse {
    #[doc = include_str!("doc/response-checktx.md")]
    CheckTx(CheckTx),
}

/// The info category of ABCI responses.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InfoResponse {
    #[doc = include_str!("doc/response-echo.md")]
    Echo(Echo),
    #[doc = include_str!("doc/response-info.md")]
    Info(Info),
    #[doc = include_str!("doc/response-query.md")]
    Query(Query),
    #[doc = include_str!("doc/response-setoption.md")]
    SetOption(SetOption),
}

/// The snapshot category of ABCI responses.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SnapshotResponse {
    #[doc = include_str!("doc/response-listsnapshots.md")]
    ListSnapshots(ListSnapshots),
    #[doc = include_str!("doc/response-offersnapshot.md")]
    OfferSnapshot(OfferSnapshot),
    #[doc = include_str!("doc/response-loadsnapshotchunk.md")]
    LoadSnapshotChunk(LoadSnapshotChunk),
    #[doc = include_str!("doc/response-applysnapshotchunk.md")]
    ApplySnapshotChunk(ApplySnapshotChunk),
}
