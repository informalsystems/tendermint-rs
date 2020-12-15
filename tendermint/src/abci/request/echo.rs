//! ABCI echo request.

use crate::Error;
use std::convert::TryFrom;
use tendermint_proto::abci::RequestEcho;
use tendermint_proto::Protobuf;

/// Request that the ABCI server echo a message back the client.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Echo {
    /// The message to echo back the client.
    pub message: String,
}

impl Echo {
    /// Constructor.
    pub fn new<S: AsRef<str>>(message: S) -> Self {
        Self {
            message: message.as_ref().to_owned(),
        }
    }
}

impl Protobuf<RequestEcho> for Echo {}

impl TryFrom<RequestEcho> for Echo {
    type Error = Error;

    fn try_from(raw: RequestEcho) -> Result<Self, Self::Error> {
        Ok(Self::new(raw.message))
    }
}

impl From<Echo> for RequestEcho {
    fn from(request: Echo) -> Self {
        Self {
            message: request.message,
        }
    }
}
