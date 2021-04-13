//! ABCI requests and request data.
//!
//! The [`Request`] enum records all possible ABCI requests. Requests that
//! contain data are modeled as a separate struct, to avoid duplication of field
//! definitions.

use crate::prelude::*;

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

use core::convert::{TryFrom, TryInto};

use super::MethodKind;

// bring into scope for doc links
#[allow(unused)]
use super::types::Snapshot;

mod apply_snapshot_chunk;
mod begin_block;
mod check_tx;
mod deliver_tx;
mod echo;
mod end_block;
mod info;
mod init_chain;
mod load_snapshot_chunk;
mod offer_snapshot;
mod query;

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
pub use query::Query;

/// All possible ABCI requests.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Request {
    #[doc = include_str!("doc/request-echo.md")]
    Echo(Echo),
    #[doc = include_str!("doc/request-flush.md")]
    Flush,
    #[doc = include_str!("doc/request-info.md")]
    Info(Info),
    #[doc = include_str!("doc/request-initchain.md")]
    InitChain(InitChain),
    #[doc = include_str!("doc/request-query.md")]
    Query(Query),
    #[doc = include_str!("doc/request-beginblock.md")]
    BeginBlock(BeginBlock),
    #[doc = include_str!("doc/request-checktx.md")]
    CheckTx(CheckTx),
    #[doc = include_str!("doc/request-delivertx.md")]
    DeliverTx(DeliverTx),
    #[doc = include_str!("doc/request-endblock.md")]
    EndBlock(EndBlock),
    #[doc = include_str!("doc/request-commit.md")]
    Commit,
    #[doc = include_str!("doc/request-listsnapshots.md")]
    ListSnapshots,
    #[doc = include_str!("doc/request-offersnapshot.md")]
    OfferSnapshot(OfferSnapshot),
    #[doc = include_str!("doc/request-loadsnapshotchunk.md")]
    LoadSnapshotChunk(LoadSnapshotChunk),
    #[doc = include_str!("doc/request-applysnapshotchunk.md")]
    ApplySnapshotChunk(ApplySnapshotChunk),
}

impl Request {
    /// Get the method kind for this request.
    pub fn kind(&self) -> MethodKind {
        use Request::*;
        match self {
            Flush => MethodKind::Flush,
            InitChain(_) => MethodKind::Consensus,
            BeginBlock(_) => MethodKind::Consensus,
            DeliverTx(_) => MethodKind::Consensus,
            EndBlock(_) => MethodKind::Consensus,
            Commit => MethodKind::Consensus,
            CheckTx(_) => MethodKind::Mempool,
            ListSnapshots => MethodKind::Snapshot,
            OfferSnapshot(_) => MethodKind::Snapshot,
            LoadSnapshotChunk(_) => MethodKind::Snapshot,
            ApplySnapshotChunk(_) => MethodKind::Snapshot,
            Info(_) => MethodKind::Info,
            Query(_) => MethodKind::Info,
            Echo(_) => MethodKind::Info,
        }
    }
}

/// The consensus category of ABCI requests.
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

impl From<ConsensusRequest> for Request {
    fn from(req: ConsensusRequest) -> Self {
        match req {
            ConsensusRequest::InitChain(x) => Self::InitChain(x),
            ConsensusRequest::BeginBlock(x) => Self::BeginBlock(x),
            ConsensusRequest::DeliverTx(x) => Self::DeliverTx(x),
            ConsensusRequest::EndBlock(x) => Self::EndBlock(x),
            ConsensusRequest::Commit => Self::Commit,
        }
    }
}

impl TryFrom<Request> for ConsensusRequest {
    type Error = &'static str;
    fn try_from(req: Request) -> Result<Self, Self::Error> {
        match req {
            Request::InitChain(x) => Ok(Self::InitChain(x)),
            Request::BeginBlock(x) => Ok(Self::BeginBlock(x)),
            Request::DeliverTx(x) => Ok(Self::DeliverTx(x)),
            Request::EndBlock(x) => Ok(Self::EndBlock(x)),
            Request::Commit => Ok(Self::Commit),
            _ => Err("wrong request type"),
        }
    }
}

/// The mempool category of ABCI requests.
#[derive(Clone, PartialEq, Debug)]
pub enum MempoolRequest {
    #[doc = include_str!("doc/request-checktx.md")]
    CheckTx(CheckTx),
}

impl From<MempoolRequest> for Request {
    fn from(req: MempoolRequest) -> Self {
        match req {
            MempoolRequest::CheckTx(x) => Self::CheckTx(x),
        }
    }
}

impl TryFrom<Request> for MempoolRequest {
    type Error = &'static str;
    fn try_from(req: Request) -> Result<Self, Self::Error> {
        match req {
            Request::CheckTx(x) => Ok(Self::CheckTx(x)),
            _ => Err("wrong request type"),
        }
    }
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
}

impl From<InfoRequest> for Request {
    fn from(req: InfoRequest) -> Self {
        match req {
            InfoRequest::Info(x) => Self::Info(x),
            InfoRequest::Query(x) => Self::Query(x),
            InfoRequest::Echo(x) => Self::Echo(x),
        }
    }
}

impl TryFrom<Request> for InfoRequest {
    type Error = &'static str;
    fn try_from(req: Request) -> Result<Self, Self::Error> {
        match req {
            Request::Info(x) => Ok(Self::Info(x)),
            Request::Query(x) => Ok(Self::Query(x)),
            Request::Echo(x) => Ok(Self::Echo(x)),
            _ => Err("wrong request type"),
        }
    }
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

impl From<SnapshotRequest> for Request {
    fn from(req: SnapshotRequest) -> Self {
        match req {
            SnapshotRequest::ListSnapshots => Self::ListSnapshots,
            SnapshotRequest::OfferSnapshot(x) => Self::OfferSnapshot(x),
            SnapshotRequest::LoadSnapshotChunk(x) => Self::LoadSnapshotChunk(x),
            SnapshotRequest::ApplySnapshotChunk(x) => Self::ApplySnapshotChunk(x),
        }
    }
}

impl TryFrom<Request> for SnapshotRequest {
    type Error = &'static str;
    fn try_from(req: Request) -> Result<Self, Self::Error> {
        match req {
            Request::ListSnapshots => Ok(Self::ListSnapshots),
            Request::OfferSnapshot(x) => Ok(Self::OfferSnapshot(x)),
            Request::LoadSnapshotChunk(x) => Ok(Self::LoadSnapshotChunk(x)),
            Request::ApplySnapshotChunk(x) => Ok(Self::ApplySnapshotChunk(x)),
            _ => Err("wrong request type"),
        }
    }
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<Request> for pb::Request {
    fn from(request: Request) -> pb::Request {
        use pb::request::Value;
        let value = match request {
            Request::Echo(x) => Some(Value::Echo(x.into())),
            Request::Flush => Some(Value::Flush(Default::default())),
            Request::Info(x) => Some(Value::Info(x.into())),
            Request::InitChain(x) => Some(Value::InitChain(x.into())),
            Request::Query(x) => Some(Value::Query(x.into())),
            Request::BeginBlock(x) => Some(Value::BeginBlock(x.into())),
            Request::CheckTx(x) => Some(Value::CheckTx(x.into())),
            Request::DeliverTx(x) => Some(Value::DeliverTx(x.into())),
            Request::EndBlock(x) => Some(Value::EndBlock(x.into())),
            Request::Commit => Some(Value::Commit(Default::default())),
            Request::ListSnapshots => Some(Value::ListSnapshots(Default::default())),
            Request::OfferSnapshot(x) => Some(Value::OfferSnapshot(x.into())),
            Request::LoadSnapshotChunk(x) => Some(Value::LoadSnapshotChunk(x.into())),
            Request::ApplySnapshotChunk(x) => Some(Value::ApplySnapshotChunk(x.into())),
        };
        pb::Request { value }
    }
}

impl TryFrom<pb::Request> for Request {
    type Error = crate::Error;

    fn try_from(request: pb::Request) -> Result<Self, Self::Error> {
        use pb::request::Value;
        match request.value {
            Some(Value::Echo(x)) => Ok(Request::Echo(x.try_into()?)),
            Some(Value::Flush(pb::RequestFlush {})) => Ok(Request::Flush),
            Some(Value::Info(x)) => Ok(Request::Info(x.try_into()?)),
            Some(Value::InitChain(x)) => Ok(Request::InitChain(x.try_into()?)),
            Some(Value::Query(x)) => Ok(Request::Query(x.try_into()?)),
            Some(Value::BeginBlock(x)) => Ok(Request::BeginBlock(x.try_into()?)),
            Some(Value::CheckTx(x)) => Ok(Request::CheckTx(x.try_into()?)),
            Some(Value::DeliverTx(x)) => Ok(Request::DeliverTx(x.try_into()?)),
            Some(Value::EndBlock(x)) => Ok(Request::EndBlock(x.try_into()?)),
            Some(Value::Commit(pb::RequestCommit {})) => Ok(Request::Commit),
            Some(Value::ListSnapshots(pb::RequestListSnapshots {})) => Ok(Request::ListSnapshots),
            Some(Value::OfferSnapshot(x)) => Ok(Request::OfferSnapshot(x.try_into()?)),
            Some(Value::LoadSnapshotChunk(x)) => Ok(Request::LoadSnapshotChunk(x.try_into()?)),
            Some(Value::ApplySnapshotChunk(x)) => Ok(Request::ApplySnapshotChunk(x.try_into()?)),
            None => Err("no request in proto".into()),
        }
    }
}

impl Protobuf<pb::Request> for Request {}
