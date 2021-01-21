//! ABCI echo response.

use crate::abci::response::{Response, ResponseInner};
use crate::{Error, Kind};
use std::convert::TryFrom;
use tendermint_proto::abci::ResponseEcho;
use tendermint_proto::Protobuf;

/// ABCI echo response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Echo {
    /// The message to be echoed back to the client.
    pub message: String,
}

impl Protobuf<ResponseEcho> for Echo {}

impl TryFrom<ResponseEcho> for Echo {
    type Error = Error;

    fn try_from(raw: ResponseEcho) -> Result<Self, Self::Error> {
        Ok(Self {
            message: raw.message,
        })
    }
}

impl From<Echo> for ResponseEcho {
    fn from(response: Echo) -> Self {
        Self {
            message: response.message,
        }
    }
}

impl ResponseInner for Echo {}

impl TryFrom<Response> for Echo {
    type Error = Error;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        match value {
            Response::Echo(res) => Ok(res),
            _ => Err(Kind::UnexpectedAbciResponseType("Echo".to_owned(), value).into()),
        }
    }
}

impl From<Echo> for Response {
    fn from(res: Echo) -> Self {
        Self::Echo(res)
    }
}
