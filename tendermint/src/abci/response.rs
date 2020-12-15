//! ABCI responses.

mod echo;
pub use echo::Echo;

use crate::{Error, Kind};
use std::convert::{TryFrom, TryInto};
use tendermint_proto::abci::response::Value;
use tendermint_proto::abci::Response as RawResponse;
use tendermint_proto::Protobuf;

/// ABCI response wrapper.
#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    /// Echo response.
    Echo(Echo),
}

impl Protobuf<RawResponse> for Response {}

impl TryFrom<RawResponse> for Response {
    type Error = Error;

    fn try_from(raw: RawResponse) -> Result<Self, Self::Error> {
        let value = raw.value.ok_or(Kind::MissingAbciResponseValue)?;
        Ok(match value {
            Value::Echo(raw_res) => Self::Echo(raw_res.try_into()?),
            _ => unimplemented!(),
            // Value::Flush(_) => {}
            // Value::Info(_) => {}
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
            }),
        }
    }
}
