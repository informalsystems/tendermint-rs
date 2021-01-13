//! ABCI responses.

mod echo;
mod info;

pub use echo::Echo;
pub use info::Info;

use crate::{Error, Kind};
use std::convert::{TryFrom, TryInto};
use tendermint_proto::abci::response::Value;
use tendermint_proto::abci::Response as RawResponse;
use tendermint_proto::Protobuf;

/// ABCI response wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    /// Echo response.
    Echo(Echo),
    /// Application info.
    Info(Info),
}

impl Protobuf<RawResponse> for Response {}

impl TryFrom<RawResponse> for Response {
    type Error = Error;

    fn try_from(raw: RawResponse) -> Result<Self, Self::Error> {
        let value = raw.value.ok_or(Kind::MissingAbciResponseValue)?;
        Ok(match value {
            Value::Echo(raw_res) => Self::Echo(raw_res.try_into()?),
            Value::Info(raw_res) => Self::Info(raw_res.try_into()?),
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

impl From<Response> for RawResponse {
    fn from(request: Response) -> Self {
        Self {
            value: Some(match request {
                Response::Echo(res) => Value::Echo(res.into()),
                Response::Info(res) => Value::Info(res.into()),
            }),
        }
    }
}

/// The inner type of a [`Response`].
pub trait ResponseInner: TryFrom<Response, Error = Error> + Into<Response> + Send {}
