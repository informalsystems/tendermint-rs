//! ABCI responses and response data.
//!
//! The [`Response`] enum records all possible ABCI responses. Responses that
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
use crate::{prelude::*, Error};

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

/// All possible ABCI responses.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Response {
    #[doc = include_str!("doc/response-exception.md")]
    Exception(Exception),
    #[doc = include_str!("doc/response-echo.md")]
    Echo(Echo),
    #[doc = include_str!("doc/response-flush.md")]
    Flush,
    #[doc = include_str!("doc/response-info.md")]
    Info(Info),
    #[doc = include_str!("doc/response-setoption.md")]
    SetOption(SetOption),
    #[doc = include_str!("doc/response-initchain.md")]
    InitChain(InitChain),
    #[doc = include_str!("doc/response-query.md")]
    Query(Query),
    #[doc = include_str!("doc/response-beginblock.md")]
    BeginBlock(BeginBlock),
    #[doc = include_str!("doc/response-checktx.md")]
    CheckTx(CheckTx),
    #[doc = include_str!("doc/response-delivertx.md")]
    DeliverTx(DeliverTx),
    #[doc = include_str!("doc/response-endblock.md")]
    EndBlock(EndBlock),
    #[doc = include_str!("doc/response-commit.md")]
    Commit(Commit),
    #[doc = include_str!("doc/response-listsnapshots.md")]
    ListSnapshots(ListSnapshots),
    #[doc = include_str!("doc/response-offersnapshot.md")]
    OfferSnapshot(OfferSnapshot),
    #[doc = include_str!("doc/response-loadsnapshotchunk.md")]
    LoadSnapshotChunk(LoadSnapshotChunk),
    #[doc = include_str!("doc/response-applysnapshotchunk.md")]
    ApplySnapshotChunk(ApplySnapshotChunk),
    #[doc = include_str!("doc/response-prepareproposal.md")]
    PrepareProposal(PrepareProposal),
    #[doc = include_str!("doc/response-processproposal.md")]
    ProcessProposal(ProcessProposal),
}

/// The consensus category of ABCI responses.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ConsensusResponse {
    #[doc = include_str!("doc/response-initchain.md")]
    InitChain(InitChain),
    #[doc = include_str!("doc/response-beginblock.md")]
    BeginBlock(BeginBlock),
    #[doc = include_str!("doc/response-delivertx.md")]
    DeliverTx(DeliverTx),
    #[doc = include_str!("doc/response-endblock.md")]
    EndBlock(EndBlock),
    #[doc = include_str!("doc/response-commit.md")]
    Commit(Commit),
}

impl From<ConsensusResponse> for Response {
    fn from(req: ConsensusResponse) -> Self {
        match req {
            ConsensusResponse::InitChain(x) => Self::InitChain(x),
            ConsensusResponse::BeginBlock(x) => Self::BeginBlock(x),
            ConsensusResponse::DeliverTx(x) => Self::DeliverTx(x),
            ConsensusResponse::EndBlock(x) => Self::EndBlock(x),
            ConsensusResponse::Commit(x) => Self::Commit(x),
        }
    }
}

impl TryFrom<Response> for ConsensusResponse {
    type Error = Error;
    fn try_from(req: Response) -> Result<Self, Self::Error> {
        match req {
            Response::InitChain(x) => Ok(Self::InitChain(x)),
            Response::BeginBlock(x) => Ok(Self::BeginBlock(x)),
            Response::DeliverTx(x) => Ok(Self::DeliverTx(x)),
            Response::EndBlock(x) => Ok(Self::EndBlock(x)),
            Response::Commit(x) => Ok(Self::Commit(x)),
            _ => Err(Error::invalid_abci_response_type()),
        }
    }
}

/// The mempool category of ABCI responses.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MempoolResponse {
    #[doc = include_str!("doc/response-checktx.md")]
    CheckTx(CheckTx),
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

impl From<InfoResponse> for Response {
    fn from(req: InfoResponse) -> Self {
        match req {
            InfoResponse::Echo(x) => Self::Echo(x),
            InfoResponse::Info(x) => Self::Info(x),
            InfoResponse::Query(x) => Self::Query(x),
            InfoResponse::SetOption(x) => Self::SetOption(x),
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
            Response::SetOption(x) => Ok(Self::SetOption(x)),
            _ => Err(Error::invalid_abci_response_type()),
        }
    }
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

mod v0_34 {
    use super::Response;
    use crate::Error;
    use tendermint_proto::v0_34::abci as pb;
    use tendermint_proto::Protobuf;

    impl From<Response> for pb::Response {
        fn from(response: Response) -> pb::Response {
            use pb::response::Value;
            let value = match response {
                Response::Exception(x) => Some(Value::Exception(x.into())),
                Response::Echo(x) => Some(Value::Echo(x.into())),
                Response::Flush => Some(Value::Flush(Default::default())),
                Response::Info(x) => Some(Value::Info(x.into())),
                Response::SetOption(x) => Some(Value::SetOption(x.into())),
                Response::InitChain(x) => Some(Value::InitChain(x.into())),
                Response::Query(x) => Some(Value::Query(x.into())),
                Response::BeginBlock(x) => Some(Value::BeginBlock(x.into())),
                Response::CheckTx(x) => Some(Value::CheckTx(x.into())),
                Response::DeliverTx(x) => Some(Value::DeliverTx(x.into())),
                Response::EndBlock(x) => Some(Value::EndBlock(x.into())),
                Response::Commit(x) => Some(Value::Commit(x.into())),
                Response::ListSnapshots(x) => Some(Value::ListSnapshots(x.into())),
                Response::OfferSnapshot(x) => Some(Value::OfferSnapshot(x.into())),
                Response::LoadSnapshotChunk(x) => Some(Value::LoadSnapshotChunk(x.into())),
                Response::ApplySnapshotChunk(x) => Some(Value::ApplySnapshotChunk(x.into())),
                Response::PrepareProposal(_) => {
                    panic!("PrepareProposal should not be used with Tendermint 0.34")
                },
                Response::ProcessProposal(_) => {
                    panic!("ProcessProposal should not be used with Tendermint 0.34")
                },
            };
            pb::Response { value }
        }
    }

    impl TryFrom<pb::Response> for Response {
        type Error = Error;

        fn try_from(response: pb::Response) -> Result<Self, Self::Error> {
            use pb::response::Value;
            match response.value {
                Some(Value::Exception(x)) => Ok(Response::Exception(x.try_into()?)),
                Some(Value::Echo(x)) => Ok(Response::Echo(x.try_into()?)),
                Some(Value::Flush(_)) => Ok(Response::Flush),
                Some(Value::Info(x)) => Ok(Response::Info(x.try_into()?)),
                Some(Value::SetOption(x)) => Ok(Response::SetOption(x.try_into()?)),
                Some(Value::InitChain(x)) => Ok(Response::InitChain(x.try_into()?)),
                Some(Value::Query(x)) => Ok(Response::Query(x.try_into()?)),
                Some(Value::BeginBlock(x)) => Ok(Response::BeginBlock(x.try_into()?)),
                Some(Value::CheckTx(x)) => Ok(Response::CheckTx(x.try_into()?)),
                Some(Value::DeliverTx(x)) => Ok(Response::DeliverTx(x.try_into()?)),
                Some(Value::EndBlock(x)) => Ok(Response::EndBlock(x.try_into()?)),
                Some(Value::Commit(x)) => Ok(Response::Commit(x.try_into()?)),
                Some(Value::ListSnapshots(x)) => Ok(Response::ListSnapshots(x.try_into()?)),
                Some(Value::OfferSnapshot(x)) => Ok(Response::OfferSnapshot(x.try_into()?)),
                Some(Value::LoadSnapshotChunk(x)) => Ok(Response::LoadSnapshotChunk(x.try_into()?)),
                Some(Value::ApplySnapshotChunk(x)) => {
                    Ok(Response::ApplySnapshotChunk(x.try_into()?))
                },
                None => Err(crate::Error::missing_data()),
            }
        }
    }

    impl Protobuf<pb::Response> for Response {}
}

mod v0_37 {
    use super::Response;
    use crate::Error;
    use tendermint_proto::v0_37::abci as pb;
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
                Response::BeginBlock(x) => Some(Value::BeginBlock(x.into())),
                Response::CheckTx(x) => Some(Value::CheckTx(x.into())),
                Response::DeliverTx(x) => Some(Value::DeliverTx(x.into())),
                Response::EndBlock(x) => Some(Value::EndBlock(x.into())),
                Response::Commit(x) => Some(Value::Commit(x.into())),
                Response::ListSnapshots(x) => Some(Value::ListSnapshots(x.into())),
                Response::OfferSnapshot(x) => Some(Value::OfferSnapshot(x.into())),
                Response::LoadSnapshotChunk(x) => Some(Value::LoadSnapshotChunk(x.into())),
                Response::ApplySnapshotChunk(x) => Some(Value::ApplySnapshotChunk(x.into())),
                Response::PrepareProposal(x) => Some(Value::PrepareProposal(x.into())),
                Response::ProcessProposal(x) => Some(Value::ProcessProposal(x.into())),
                Response::SetOption(_) => {
                    panic!("SetOption should not be used with Tendermint 0.37")
                },
            };
            pb::Response { value }
        }
    }

    impl TryFrom<pb::Response> for Response {
        type Error = Error;

        fn try_from(response: pb::Response) -> Result<Self, Self::Error> {
            use pb::response::Value;
            match response.value {
                Some(Value::Exception(x)) => Ok(Response::Exception(x.try_into()?)),
                Some(Value::Echo(x)) => Ok(Response::Echo(x.try_into()?)),
                Some(Value::Flush(_)) => Ok(Response::Flush),
                Some(Value::Info(x)) => Ok(Response::Info(x.try_into()?)),
                Some(Value::InitChain(x)) => Ok(Response::InitChain(x.try_into()?)),
                Some(Value::Query(x)) => Ok(Response::Query(x.try_into()?)),
                Some(Value::BeginBlock(x)) => Ok(Response::BeginBlock(x.try_into()?)),
                Some(Value::CheckTx(x)) => Ok(Response::CheckTx(x.try_into()?)),
                Some(Value::DeliverTx(x)) => Ok(Response::DeliverTx(x.try_into()?)),
                Some(Value::EndBlock(x)) => Ok(Response::EndBlock(x.try_into()?)),
                Some(Value::Commit(x)) => Ok(Response::Commit(x.try_into()?)),
                Some(Value::ListSnapshots(x)) => Ok(Response::ListSnapshots(x.try_into()?)),
                Some(Value::OfferSnapshot(x)) => Ok(Response::OfferSnapshot(x.try_into()?)),
                Some(Value::LoadSnapshotChunk(x)) => Ok(Response::LoadSnapshotChunk(x.try_into()?)),
                Some(Value::ApplySnapshotChunk(x)) => {
                    Ok(Response::ApplySnapshotChunk(x.try_into()?))
                },
                Some(Value::PrepareProposal(x)) => Ok(Response::PrepareProposal(x.try_into()?)),
                Some(Value::ProcessProposal(x)) => Ok(Response::ProcessProposal(x.try_into()?)),
                None => Err(crate::Error::missing_data()),
            }
        }
    }

    impl Protobuf<pb::Response> for Response {}
}
