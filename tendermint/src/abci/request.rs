//! ABCI requests.

mod echo;
mod info;

pub use echo::Echo;
pub use info::Info;

use crate::abci::response::ResponseInner;
use crate::{Error, Kind};
use std::convert::{TryFrom, TryInto};
use tendermint_proto::abci::request::Value;
use tendermint_proto::abci::Request as RawRequest;
use tendermint_proto::Protobuf;

/// ABCI request wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// Request that the ABCI server echo a specific message back to the client.
    Echo(Echo),
    /// Return application info.
    Info(Info),
}

impl Protobuf<RawRequest> for Request {}

impl TryFrom<RawRequest> for Request {
    type Error = Error;

    fn try_from(raw: RawRequest) -> Result<Self, Self::Error> {
        let value = raw.value.ok_or(Kind::MissingAbciRequestValue)?;
        Ok(match value {
            Value::Echo(raw_req) => Self::Echo(raw_req.try_into()?),
            Value::Info(raw_req) => Self::Info(raw_req.try_into()?),
            _ => unimplemented!(),
            // Value::Flush(_) => {}
            // Value::SetOption(_) => {}
            // Value::InitChain(_) => {}
            // Value::Query(_) => {}
            // Value::BeginBlock(_) => {}
            // Value::CheckTx(_) => {}
            // Value::DeliverTx(_) => {}
            // Value::EndBlock(_) => {}
            // Value::Commit(_) => {}
            // Value::ListSnapshots(_) => {}
            // Value::OfferSnapshot(_) => {}
            // Value::LoadSnapshotChunk(_) => {}
            // Value::ApplySnapshotChunk(_) => {}
        })
    }
}

impl From<Request> for RawRequest {
    fn from(request: Request) -> Self {
        Self {
            value: Some(match request {
                Request::Echo(req) => Value::Echo(req.into()),
                Request::Info(req) => Value::Info(req.into()),
            }),
        }
    }
}

/// The inner type of a [`Request`].
pub trait RequestInner: TryFrom<Request, Error = Error> + Into<Request> + Send {
    /// The corresponding response type for this request.
    type Response: ResponseInner;
}
