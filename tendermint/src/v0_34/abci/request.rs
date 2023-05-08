use tendermint_proto::v0_34::abci as pb;
use tendermint_proto::Protobuf;

use crate::abci::MethodKind;
use crate::Error;

pub use crate::abci::request::{
    ApplySnapshotChunk, BeginBlock, CheckTx, CheckTxKind, DeliverTx, Echo, EndBlock, Info,
    InitChain, LoadSnapshotChunk, OfferSnapshot, Query, SetOption,
};

/// All possible ABCI requests in CometBFT 0.34.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Request {
    #[doc = include_str!("../../abci/doc/request-echo.md")]
    Echo(Echo),
    #[doc = include_str!("../../abci/doc/request-flush.md")]
    Flush,
    #[doc = include_str!("../../abci/doc/request-info.md")]
    Info(Info),
    #[doc = include_str!("../../abci/doc/request-setoption.md")]
    SetOption(SetOption),
    #[doc = include_str!("../../abci/doc/request-initchain.md")]
    InitChain(InitChain),
    #[doc = include_str!("../../abci/doc/request-query.md")]
    Query(Query),
    #[doc = include_str!("../../abci/doc/request-beginblock.md")]
    BeginBlock(BeginBlock),
    #[doc = include_str!("../../abci/doc/request-checktx.md")]
    CheckTx(CheckTx),
    #[doc = include_str!("../../abci/doc/request-delivertx.md")]
    DeliverTx(DeliverTx),
    #[doc = include_str!("../../abci/doc/request-endblock.md")]
    EndBlock(EndBlock),
    #[doc = include_str!("../../abci/doc/request-commit.md")]
    Commit,
    #[doc = include_str!("../../abci/doc/request-listsnapshots.md")]
    ListSnapshots,
    #[doc = include_str!("../../abci/doc/request-offersnapshot.md")]
    OfferSnapshot(OfferSnapshot),
    #[doc = include_str!("../../abci/doc/request-loadsnapshotchunk.md")]
    LoadSnapshotChunk(LoadSnapshotChunk),
    #[doc = include_str!("../../abci/doc/request-applysnapshotchunk.md")]
    ApplySnapshotChunk(ApplySnapshotChunk),
}

/// The consensus category of ABCI requests.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ConsensusRequest {
    #[doc = include_str!("../../abci/doc/request-initchain.md")]
    InitChain(InitChain),
    #[doc = include_str!("../../abci/doc/request-beginblock.md")]
    BeginBlock(BeginBlock),
    #[doc = include_str!("../../abci/doc/request-delivertx.md")]
    DeliverTx(DeliverTx),
    #[doc = include_str!("../../abci/doc/request-endblock.md")]
    EndBlock(EndBlock),
    #[doc = include_str!("../../abci/doc/request-commit.md")]
    Commit,
}

/// The mempool category of ABCI requests.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MempoolRequest {
    #[doc = include_str!("../../abci/doc/request-checktx.md")]
    CheckTx(CheckTx),
}

/// The info category of ABCI requests.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InfoRequest {
    #[doc = include_str!("../../abci/doc/request-info.md")]
    Info(Info),
    #[doc = include_str!("../../abci/doc/request-query.md")]
    Query(Query),
    #[doc = include_str!("../../abci/doc/request-echo.md")]
    Echo(Echo),
    #[doc = include_str!("../../abci/doc/request-setoption.md")]
    SetOption(SetOption),
}

/// The snapshot category of ABCI requests.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SnapshotRequest {
    #[doc = include_str!("../../abci/doc/request-listsnapshots.md")]
    ListSnapshots,
    #[doc = include_str!("../../abci/doc/request-offersnapshot.md")]
    OfferSnapshot(OfferSnapshot),
    #[doc = include_str!("../../abci/doc/request-loadsnapshotchunk.md")]
    LoadSnapshotChunk(LoadSnapshotChunk),
    #[doc = include_str!("../../abci/doc/request-applysnapshotchunk.md")]
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
            SetOption(_) => MethodKind::Info,
        }
    }
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
    type Error = Error;
    fn try_from(req: Request) -> Result<Self, Self::Error> {
        match req {
            Request::InitChain(x) => Ok(Self::InitChain(x)),
            Request::BeginBlock(x) => Ok(Self::BeginBlock(x)),
            Request::DeliverTx(x) => Ok(Self::DeliverTx(x)),
            Request::EndBlock(x) => Ok(Self::EndBlock(x)),
            Request::Commit => Ok(Self::Commit),
            _ => Err(Error::invalid_abci_request_type()),
        }
    }
}

impl From<MempoolRequest> for Request {
    fn from(req: MempoolRequest) -> Self {
        match req {
            MempoolRequest::CheckTx(x) => Self::CheckTx(x),
        }
    }
}

impl TryFrom<Request> for MempoolRequest {
    type Error = Error;
    fn try_from(req: Request) -> Result<Self, Self::Error> {
        match req {
            Request::CheckTx(x) => Ok(Self::CheckTx(x)),
            _ => Err(Error::invalid_abci_request_type()),
        }
    }
}

impl From<InfoRequest> for Request {
    fn from(req: InfoRequest) -> Self {
        match req {
            InfoRequest::Info(x) => Self::Info(x),
            InfoRequest::Query(x) => Self::Query(x),
            InfoRequest::Echo(x) => Self::Echo(x),
            InfoRequest::SetOption(x) => Self::SetOption(x),
        }
    }
}

impl TryFrom<Request> for InfoRequest {
    type Error = Error;
    fn try_from(req: Request) -> Result<Self, Self::Error> {
        match req {
            Request::Info(x) => Ok(Self::Info(x)),
            Request::Query(x) => Ok(Self::Query(x)),
            Request::Echo(x) => Ok(Self::Echo(x)),
            Request::SetOption(x) => Ok(Self::SetOption(x)),
            _ => Err(Error::invalid_abci_request_type()),
        }
    }
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
    type Error = Error;
    fn try_from(req: Request) -> Result<Self, Self::Error> {
        match req {
            Request::ListSnapshots => Ok(Self::ListSnapshots),
            Request::OfferSnapshot(x) => Ok(Self::OfferSnapshot(x)),
            Request::LoadSnapshotChunk(x) => Ok(Self::LoadSnapshotChunk(x)),
            Request::ApplySnapshotChunk(x) => Ok(Self::ApplySnapshotChunk(x)),
            _ => Err(Error::invalid_abci_request_type()),
        }
    }
}

// =============================================================================
// Protobuf conversions
// =============================================================================

impl From<Request> for pb::Request {
    fn from(request: Request) -> pb::Request {
        use pb::request::Value;
        let value = match request {
            Request::Echo(x) => Some(Value::Echo(x.into())),
            Request::Flush => Some(Value::Flush(Default::default())),
            Request::Info(x) => Some(Value::Info(x.into())),
            Request::SetOption(x) => Some(Value::SetOption(x.into())),
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
    type Error = Error;

    fn try_from(request: pb::Request) -> Result<Self, Self::Error> {
        use pb::request::Value;
        match request.value {
            Some(Value::Echo(x)) => Ok(Request::Echo(x.try_into()?)),
            Some(Value::Flush(pb::RequestFlush {})) => Ok(Request::Flush),
            Some(Value::Info(x)) => Ok(Request::Info(x.try_into()?)),
            Some(Value::SetOption(x)) => Ok(Request::SetOption(x.try_into()?)),
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
            None => Err(crate::Error::missing_data()),
        }
    }
}

impl Protobuf<pb::Request> for Request {}
