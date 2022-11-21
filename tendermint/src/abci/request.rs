//! ABCI requests and request data.
//!
//! The [`Request`] enum records all possible ABCI requests. Requests that
//! contain data are modeled as a separate struct, to avoid duplication of field
//! definitions.

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

pub(super) mod apply_snapshot_chunk;
pub(super) mod begin_block;
pub(super) mod check_tx;
pub(super) mod deliver_tx;
pub(super) mod echo;
pub(super) mod end_block;
pub(super) mod info;
pub(super) mod init_chain;
pub(super) mod load_snapshot_chunk;
pub(super) mod offer_snapshot;
pub(super) mod prepare_proposal;
pub(super) mod process_proposal;
pub(super) mod query;
pub(super) mod set_option;

pub use apply_snapshot_chunk::ApplySnapshotChunk;
pub use begin_block::BeginBlock;
pub use check_tx::{CheckTx, CheckTxKind};
pub use deliver_tx::DeliverTx;
pub use echo::Echo;
pub use end_block::EndBlock;
pub use info::Info;
pub use init_chain::InitChain;
pub use load_snapshot_chunk::LoadSnapshotChunk;
pub use offer_snapshot::OfferSnapshot;
pub use prepare_proposal::PrepareProposal;
pub use process_proposal::ProcessProposal;
pub use query::Query;
pub use set_option::SetOption;

/// The consensus category of ABCI requests.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ConsensusRequest {
    #[doc = include_str!("doc/request-initchain.md")]
    InitChain(InitChain),
    #[doc = include_str!("doc/request-beginblock.md")]
    BeginBlock(BeginBlock),
    #[doc = include_str!("doc/request-delivertx.md")]
    DeliverTx(DeliverTx),
    #[doc = include_str!("doc/request-endblock.md")]
    EndBlock(EndBlock),
    #[doc = include_str!("doc/request-commit.md")]
    Commit,
}

/// The mempool category of ABCI requests.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MempoolRequest {
    #[doc = include_str!("doc/request-checktx.md")]
    CheckTx(CheckTx),
}

/// The info category of ABCI requests.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InfoRequest {
    #[doc = include_str!("doc/request-info.md")]
    Info(Info),
    #[doc = include_str!("doc/request-query.md")]
    Query(Query),
    #[doc = include_str!("doc/request-echo.md")]
    Echo(Echo),
    #[doc = include_str!("doc/request-setoption.md")]
    SetOption(SetOption),
}

/// The snapshot category of ABCI requests.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SnapshotRequest {
    #[doc = include_str!("doc/request-listsnapshots.md")]
    ListSnapshots,
    #[doc = include_str!("doc/request-offersnapshot.md")]
    OfferSnapshot(OfferSnapshot),
    #[doc = include_str!("doc/request-loadsnapshotchunk.md")]
    LoadSnapshotChunk(LoadSnapshotChunk),
    #[doc = include_str!("doc/request-applysnapshotchunk.md")]
    ApplySnapshotChunk(ApplySnapshotChunk),
}
