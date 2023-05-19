use tendermint_proto::v0_38::abci as pb;
use tendermint_proto::Protobuf;

use crate::abci::MethodKind;
use crate::Error;

pub use crate::abci::request::{
    ApplySnapshotChunk, CheckTx, CheckTxKind, Echo, ExtendVote, FinalizeBlock, Info, InitChain,
    LoadSnapshotChunk, OfferSnapshot, PrepareProposal, ProcessProposal, Query, VerifyVoteExtension,
};

/// All possible ABCI requests in CometBFT 0.38.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Request {
    #[doc = include_str!("../../abci/doc/request-echo.md")]
    Echo(Echo),
    #[doc = include_str!("../../abci/doc/request-flush.md")]
    Flush,
    #[doc = include_str!("../../abci/doc/request-info.md")]
    Info(Info),
    #[doc = include_str!("../../abci/doc/request-initchain.md")]
    InitChain(InitChain),
    #[doc = include_str!("../../abci/doc/request-query.md")]
    Query(Query),
    #[doc = include_str!("../../abci/doc/request-checktx.md")]
    CheckTx(CheckTx),
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
    #[doc = include_str!("../../abci/doc/request-prepareproposal.md")]
    PrepareProposal(PrepareProposal),
    #[doc = include_str!("../../abci/doc/request-processproposal.md")]
    ProcessProposal(ProcessProposal),
    #[doc = include_str!("../../abci/doc/request-extendvote.md")]
    ExtendVote(ExtendVote),
    #[doc = include_str!("../../abci/doc/request-verifyvoteextension.md")]
    VerifyVoteExtension(VerifyVoteExtension),
    #[doc = include_str!("../../abci/doc/request-finalizeblock.md")]
    FinalizeBlock(FinalizeBlock),
}

/// The consensus category of ABCI requests.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ConsensusRequest {
    #[doc = include_str!("../../abci/doc/request-initchain.md")]
    InitChain(InitChain),
    #[doc = include_str!("../../abci/doc/request-prepareproposal.md")]
    PrepareProposal(PrepareProposal),
    #[doc = include_str!("../../abci/doc/request-processproposal.md")]
    ProcessProposal(ProcessProposal),
    #[doc = include_str!("../../abci/doc/request-commit.md")]
    Commit,
    #[doc = include_str!("../../abci/doc/request-extendvote.md")]
    ExtendVote(ExtendVote),
    #[doc = include_str!("../../abci/doc/request-verifyvoteextension.md")]
    VerifyVoteExtension(VerifyVoteExtension),
    #[doc = include_str!("../../abci/doc/request-finalizeblock.md")]
    FinalizeBlock(FinalizeBlock),
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
            Commit => MethodKind::Consensus,
            PrepareProposal(_) => MethodKind::Consensus,
            ProcessProposal(_) => MethodKind::Consensus,
            ExtendVote(_) => MethodKind::Consensus,
            VerifyVoteExtension(_) => MethodKind::Consensus,
            FinalizeBlock(_) => MethodKind::Consensus,
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

impl From<ConsensusRequest> for Request {
    fn from(req: ConsensusRequest) -> Self {
        match req {
            ConsensusRequest::InitChain(x) => Self::InitChain(x),
            ConsensusRequest::PrepareProposal(x) => Self::PrepareProposal(x),
            ConsensusRequest::ProcessProposal(x) => Self::ProcessProposal(x),
            ConsensusRequest::Commit => Self::Commit,
            ConsensusRequest::ExtendVote(x) => Self::ExtendVote(x),
            ConsensusRequest::VerifyVoteExtension(x) => Self::VerifyVoteExtension(x),
            ConsensusRequest::FinalizeBlock(x) => Self::FinalizeBlock(x),
        }
    }
}

impl TryFrom<Request> for ConsensusRequest {
    type Error = Error;
    fn try_from(req: Request) -> Result<Self, Self::Error> {
        match req {
            Request::InitChain(x) => Ok(Self::InitChain(x)),
            Request::PrepareProposal(x) => Ok(Self::PrepareProposal(x)),
            Request::ProcessProposal(x) => Ok(Self::ProcessProposal(x)),
            Request::Commit => Ok(Self::Commit),
            Request::ExtendVote(x) => Ok(Self::ExtendVote(x)),
            Request::VerifyVoteExtension(x) => Ok(Self::VerifyVoteExtension(x)),
            Request::FinalizeBlock(x) => Ok(Self::FinalizeBlock(x)),
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
            Request::InitChain(x) => Some(Value::InitChain(x.into())),
            Request::Query(x) => Some(Value::Query(x.into())),
            Request::CheckTx(x) => Some(Value::CheckTx(x.into())),
            Request::Commit => Some(Value::Commit(Default::default())),
            Request::ListSnapshots => Some(Value::ListSnapshots(Default::default())),
            Request::OfferSnapshot(x) => Some(Value::OfferSnapshot(x.into())),
            Request::LoadSnapshotChunk(x) => Some(Value::LoadSnapshotChunk(x.into())),
            Request::ApplySnapshotChunk(x) => Some(Value::ApplySnapshotChunk(x.into())),
            Request::PrepareProposal(x) => Some(Value::PrepareProposal(x.into())),
            Request::ProcessProposal(x) => Some(Value::ProcessProposal(x.into())),
            Request::ExtendVote(x) => Some(Value::ExtendVote(x.into())),
            Request::VerifyVoteExtension(x) => Some(Value::VerifyVoteExtension(x.into())),
            Request::FinalizeBlock(x) => Some(Value::FinalizeBlock(x.into())),
        };
        pb::Request { value }
    }
}

impl TryFrom<pb::Request> for Request {
    type Error = Error;

    fn try_from(request: pb::Request) -> Result<Self, Self::Error> {
        use pb::request::Value;

        let value = request.value.ok_or_else(Error::missing_data)?;
        let request = match value {
            Value::Echo(x) => Request::Echo(x.try_into()?),
            Value::Flush(pb::RequestFlush {}) => Request::Flush,
            Value::Info(x) => Request::Info(x.try_into()?),
            Value::InitChain(x) => Request::InitChain(x.try_into()?),
            Value::Query(x) => Request::Query(x.try_into()?),
            Value::CheckTx(x) => Request::CheckTx(x.try_into()?),
            Value::Commit(pb::RequestCommit {}) => Request::Commit,
            Value::ListSnapshots(pb::RequestListSnapshots {}) => Request::ListSnapshots,
            Value::OfferSnapshot(x) => Request::OfferSnapshot(x.try_into()?),
            Value::LoadSnapshotChunk(x) => Request::LoadSnapshotChunk(x.try_into()?),
            Value::ApplySnapshotChunk(x) => Request::ApplySnapshotChunk(x.try_into()?),
            Value::PrepareProposal(x) => Request::PrepareProposal(x.try_into()?),
            Value::ProcessProposal(x) => Request::ProcessProposal(x.try_into()?),
            Value::ExtendVote(x) => Request::ExtendVote(x.try_into()?),
            Value::VerifyVoteExtension(x) => Request::VerifyVoteExtension(x.try_into()?),
            Value::FinalizeBlock(x) => Request::FinalizeBlock(x.try_into()?),
        };
        Ok(request)
    }
}

impl Protobuf<pb::Request> for Request {}
