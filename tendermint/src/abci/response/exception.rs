use crate::prelude::*;

#[doc = include_str!("../doc/response-exception.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Exception {
    /// Undocumented.
    pub error: String,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

use core::convert::TryFrom;
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<Exception> for pb::ResponseException {
    fn from(exception: Exception) -> Self {
        Self {
            error: exception.error,
        }
    }
}

impl TryFrom<pb::ResponseException> for Exception {
    type Error = crate::Error;

    fn try_from(exception: pb::ResponseException) -> Result<Self, Self::Error> {
        Ok(Self {
            error: exception.error,
        })
    }
}

impl Protobuf<pb::ResponseException> for Exception {}
