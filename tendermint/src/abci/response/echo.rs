use crate::prelude::*;

#[doc = include_str!("../doc/response-echo.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Echo {
    /// The message sent in the request.
    pub message: String,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

use core::convert::TryFrom;

use tendermint_proto::{abci as pb, Protobuf};

impl From<Echo> for pb::ResponseEcho {
    fn from(echo: Echo) -> Self {
        Self {
            message: echo.message,
        }
    }
}

impl TryFrom<pb::ResponseEcho> for Echo {
    type Error = crate::Error;

    fn try_from(echo: pb::ResponseEcho) -> Result<Self, Self::Error> {
        Ok(Self {
            message: echo.message,
        })
    }
}

impl Protobuf<pb::ResponseEcho> for Echo {}
