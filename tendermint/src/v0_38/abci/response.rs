pub use crate::abci::response::{
    ApplySnapshotChunk, BeginBlock, CheckTx, Commit, DeliverTx, Echo, EndBlock, Exception,
    ExtendVote, FinalizeBlock, Info, InitChain, ListSnapshots, LoadSnapshotChunk, OfferSnapshot,
    PrepareProposal, ProcessProposal, Query, VerifyVoteExtension,
};
use crate::Error;

/// All possible ABCI responses for this protocol version.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Response {
    #[doc = include_str!("../../abci/doc/response-exception.md")]
    Exception(Exception),
    #[doc = include_str!("../../abci/doc/response-echo.md")]
    Echo(Echo),
    #[doc = include_str!("../../abci/doc/response-flush.md")]
    Flush,
    #[doc = include_str!("../../abci/doc/response-info.md")]
    Info(Info),
    #[doc = include_str!("../../abci/doc/response-initchain.md")]
    InitChain(InitChain),
    #[doc = include_str!("../../abci/doc/response-query.md")]
    Query(Query),
    #[doc = include_str!("../../abci/doc/response-checktx.md")]
    CheckTx(CheckTx),
    #[doc = include_str!("../../abci/doc/response-commit.md")]
    Commit(Commit),
    #[doc = include_str!("../../abci/doc/response-listsnapshots.md")]
    ListSnapshots(ListSnapshots),
    #[doc = include_str!("../../abci/doc/response-offersnapshot.md")]
    OfferSnapshot(OfferSnapshot),
    #[doc = include_str!("../../abci/doc/response-loadsnapshotchunk.md")]
    LoadSnapshotChunk(LoadSnapshotChunk),
    #[doc = include_str!("../../abci/doc/response-applysnapshotchunk.md")]
    ApplySnapshotChunk(ApplySnapshotChunk),
    #[doc = include_str!("../../abci/doc/response-prepareproposal.md")]
    PrepareProposal(PrepareProposal),
    #[doc = include_str!("../../abci/doc/response-processproposal.md")]
    ProcessProposal(ProcessProposal),
    #[doc = include_str!("../../abci/doc/response-extendvote.md")]
    ExtendVote(ExtendVote),
    #[doc = include_str!("../../abci/doc/response-verifyvoteextension.md")]
    VerifyVoteExtension(VerifyVoteExtension),
    #[doc = include_str!("../../abci/doc/response-finalizeblock.md")]
    FinalizeBlock(FinalizeBlock),
}

/// The consensus category of ABCI responses.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ConsensusResponse {
    #[doc = include_str!("../../abci/doc/response-initchain.md")]
    InitChain(InitChain),
    #[doc = include_str!("../../abci/doc/response-prepareproposal.md")]
    PrepareProposal(PrepareProposal),
    #[doc = include_str!("../../abci/doc/response-processproposal.md")]
    ProcessProposal(ProcessProposal),
    #[doc = include_str!("../../abci/doc/response-commit.md")]
    Commit(Commit),
    #[doc = include_str!("../../abci/doc/response-extendvote.md")]
    ExtendVote(ExtendVote),
    #[doc = include_str!("../../abci/doc/response-verifyvoteextension.md")]
    VerifyVoteExtension(VerifyVoteExtension),
    #[doc = include_str!("../../abci/doc/response-finalizeblock.md")]
    FinalizeBlock(FinalizeBlock),
}

/// The mempool category of ABCI responses.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MempoolResponse {
    #[doc = include_str!("../../abci/doc/response-checktx.md")]
    CheckTx(CheckTx),
}

/// The info category of ABCI responses.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InfoResponse {
    #[doc = include_str!("../../abci/doc/response-echo.md")]
    Echo(Echo),
    #[doc = include_str!("../../abci/doc/response-info.md")]
    Info(Info),
    #[doc = include_str!("../../abci/doc/response-query.md")]
    Query(Query),
}

/// The snapshot category of ABCI responses.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SnapshotResponse {
    #[doc = include_str!("../../abci/doc/response-listsnapshots.md")]
    ListSnapshots(ListSnapshots),
    #[doc = include_str!("../../abci/doc/response-offersnapshot.md")]
    OfferSnapshot(OfferSnapshot),
    #[doc = include_str!("../../abci/doc/response-loadsnapshotchunk.md")]
    LoadSnapshotChunk(LoadSnapshotChunk),
    #[doc = include_str!("../../abci/doc/response-applysnapshotchunk.md")]
    ApplySnapshotChunk(ApplySnapshotChunk),
}

impl From<ConsensusResponse> for Response {
    fn from(req: ConsensusResponse) -> Self {
        match req {
            ConsensusResponse::InitChain(x) => Self::InitChain(x),
            ConsensusResponse::PrepareProposal(x) => Self::PrepareProposal(x),
            ConsensusResponse::ProcessProposal(x) => Self::ProcessProposal(x),
            ConsensusResponse::Commit(x) => Self::Commit(x),
            ConsensusResponse::ExtendVote(x) => Self::ExtendVote(x),
            ConsensusResponse::VerifyVoteExtension(x) => Self::VerifyVoteExtension(x),
            ConsensusResponse::FinalizeBlock(x) => Self::FinalizeBlock(x),
        }
    }
}

impl TryFrom<Response> for ConsensusResponse {
    type Error = Error;
    fn try_from(req: Response) -> Result<Self, Self::Error> {
        match req {
            Response::InitChain(x) => Ok(Self::InitChain(x)),
            Response::PrepareProposal(x) => Ok(Self::PrepareProposal(x)),
            Response::ProcessProposal(x) => Ok(Self::ProcessProposal(x)),
            Response::Commit(x) => Ok(Self::Commit(x)),
            Response::ExtendVote(x) => Ok(Self::ExtendVote(x)),
            Response::VerifyVoteExtension(x) => Ok(Self::VerifyVoteExtension(x)),
            Response::FinalizeBlock(x) => Ok(Self::FinalizeBlock(x)),
            _ => Err(Error::invalid_abci_response_type()),
        }
    }
}

impl From<MempoolResponse> for Response {
    fn from(req: MempoolResponse) -> Self {
        match req {
            MempoolResponse::CheckTx(x) => Self::CheckTx(x),
        }
    }
}

impl TryFrom<Response> for MempoolResponse {
    type Error = Error;
    fn try_from(req: Response) -> Result<Self, Self::Error> {
        match req {
            Response::CheckTx(x) => Ok(Self::CheckTx(x)),
            _ => Err(Error::invalid_abci_response_type()),
        }
    }
}

impl From<InfoResponse> for Response {
    fn from(req: InfoResponse) -> Self {
        match req {
            InfoResponse::Echo(x) => Self::Echo(x),
            InfoResponse::Info(x) => Self::Info(x),
            InfoResponse::Query(x) => Self::Query(x),
        }
    }
}

impl TryFrom<Response> for InfoResponse {
    type Error = Error;
    fn try_from(req: Response) -> Result<Self, Self::Error> {
        match req {
            Response::Echo(x) => Ok(Self::Echo(x)),
            Response::Info(x) => Ok(Self::Info(x)),
            Response::Query(x) => Ok(Self::Query(x)),
            _ => Err(Error::invalid_abci_response_type()),
        }
    }
}

impl From<SnapshotResponse> for Response {
    fn from(req: SnapshotResponse) -> Self {
        match req {
            SnapshotResponse::ListSnapshots(x) => Self::ListSnapshots(x),
            SnapshotResponse::OfferSnapshot(x) => Self::OfferSnapshot(x),
            SnapshotResponse::LoadSnapshotChunk(x) => Self::LoadSnapshotChunk(x),
            SnapshotResponse::ApplySnapshotChunk(x) => Self::ApplySnapshotChunk(x),
        }
    }
}

impl TryFrom<Response> for SnapshotResponse {
    type Error = Error;
    fn try_from(req: Response) -> Result<Self, Self::Error> {
        match req {
            Response::ListSnapshots(x) => Ok(Self::ListSnapshots(x)),
            Response::OfferSnapshot(x) => Ok(Self::OfferSnapshot(x)),
            Response::LoadSnapshotChunk(x) => Ok(Self::LoadSnapshotChunk(x)),
            Response::ApplySnapshotChunk(x) => Ok(Self::ApplySnapshotChunk(x)),
            _ => Err(Error::invalid_abci_response_type()),
        }
    }
}

// =============================================================================
// Protobuf conversions
// =============================================================================

use tendermint_proto::v0_38::abci as pb;
use tendermint_proto::Protobuf;

impl From<Response> for pb::Response {
    fn from(response: Response) -> pb::Response {
        use pb::response::Value;
        let value = match response {
            Response::Exception(x) => Some(Value::Exception(x.into())),
            Response::Echo(x) => Some(Value::Echo(x.into())),
            Response::Flush => Some(Value::Flush(Default::default())),
            Response::Info(x) => Some(Value::Info(x.into())),
            Response::InitChain(x) => Some(Value::InitChain(x.into())),
            Response::Query(x) => Some(Value::Query(x.into())),
            Response::CheckTx(x) => Some(Value::CheckTx(x.into())),
            Response::Commit(x) => Some(Value::Commit(x.into())),
            Response::ListSnapshots(x) => Some(Value::ListSnapshots(x.into())),
            Response::OfferSnapshot(x) => Some(Value::OfferSnapshot(x.into())),
            Response::LoadSnapshotChunk(x) => Some(Value::LoadSnapshotChunk(x.into())),
            Response::ApplySnapshotChunk(x) => Some(Value::ApplySnapshotChunk(x.into())),
            Response::PrepareProposal(x) => Some(Value::PrepareProposal(x.into())),
            Response::ProcessProposal(x) => Some(Value::ProcessProposal(x.into())),
            Response::ExtendVote(x) => Some(Value::ExtendVote(x.into())),
            Response::VerifyVoteExtension(x) => Some(Value::VerifyVoteExtension(x.into())),
            Response::FinalizeBlock(x) => Some(Value::FinalizeBlock(x.into())),
        };
        pb::Response { value }
    }
}

impl TryFrom<pb::Response> for Response {
    type Error = Error;

    fn try_from(response: pb::Response) -> Result<Self, Self::Error> {
        use pb::response::Value;

        let value = response.value.ok_or_else(Error::missing_data)?;

        let response = match value {
            Value::Exception(x) => Response::Exception(x.try_into()?),
            Value::Echo(x) => Response::Echo(x.try_into()?),
            Value::Flush(_) => Response::Flush,
            Value::Info(x) => Response::Info(x.try_into()?),
            Value::InitChain(x) => Response::InitChain(x.try_into()?),
            Value::Query(x) => Response::Query(x.try_into()?),
            Value::CheckTx(x) => Response::CheckTx(x.try_into()?),
            Value::Commit(x) => Response::Commit(x.try_into()?),
            Value::ListSnapshots(x) => Response::ListSnapshots(x.try_into()?),
            Value::OfferSnapshot(x) => Response::OfferSnapshot(x.try_into()?),
            Value::LoadSnapshotChunk(x) => Response::LoadSnapshotChunk(x.try_into()?),
            Value::ApplySnapshotChunk(x) => Response::ApplySnapshotChunk(x.try_into()?),
            Value::PrepareProposal(x) => Response::PrepareProposal(x.try_into()?),
            Value::ProcessProposal(x) => Response::ProcessProposal(x.try_into()?),
            Value::ExtendVote(x) => Response::ExtendVote(x.try_into()?),
            Value::VerifyVoteExtension(x) => Response::VerifyVoteExtension(x.try_into()?),
            Value::FinalizeBlock(x) => Response::FinalizeBlock(x.try_into()?),
        };
        Ok(response)
    }
}

impl Protobuf<pb::Response> for Response {}
