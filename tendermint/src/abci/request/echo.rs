//! ABCI echo request.

use crate::abci::request::{Request, RequestInner};
use crate::abci::response;
use crate::{Error, Kind};
use std::convert::TryFrom;
use tendermint_proto::abci::RequestEcho;
use tendermint_proto::Protobuf;

/// Request that the ABCI server echo a message back the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Echo {
    /// The message to echo back the client.
    pub message: String,
}

impl Protobuf<RequestEcho> for Echo {}

impl TryFrom<RequestEcho> for Echo {
    type Error = Error;

    fn try_from(raw: RequestEcho) -> Result<Self, Self::Error> {
        Ok(Self {
            message: raw.message,
        })
    }
}

impl From<Echo> for RequestEcho {
    fn from(request: Echo) -> Self {
        Self {
            message: request.message,
        }
    }
}

impl RequestInner for Echo {
    type Response = response::Echo;
}

impl TryFrom<Request> for Echo {
    type Error = Error;

    fn try_from(value: Request) -> Result<Self, Self::Error> {
        match value {
            Request::Echo(r) => Ok(r),
            _ => Err(Kind::UnexpectedAbciRequestType("Echo".to_owned(), value).into()),
        }
    }
}

impl From<Echo> for Request {
    fn from(req: Echo) -> Self {
        Self::Echo(req)
    }
}
