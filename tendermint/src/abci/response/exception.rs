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

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

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
    type Error = &'static str;

    fn try_from(exception: pb::ResponseException) -> Result<Self, Self::Error> {
        Ok(Self {
            error: exception.error,
        })
    }
}

impl Protobuf<pb::ResponseException> for Exception {}
