//! ABCI echo response.

use crate::Error;
use std::convert::TryFrom;
use tendermint_proto::abci::ResponseEcho;
use tendermint_proto::Protobuf;

/// ABCI echo response.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Echo {
    /// The message to be echoed back to the client.
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

impl Protobuf<ResponseEcho> for Echo {}

impl TryFrom<ResponseEcho> for Echo {
    type Error = Error;

    fn try_from(raw: ResponseEcho) -> Result<Self, Self::Error> {
        Ok(Self::new(raw.message))
    }
}

impl From<Echo> for ResponseEcho {
    fn from(response: Echo) -> Self {
        Self {
            message: response.message,
        }
    }
}
