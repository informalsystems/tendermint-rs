use crate::prelude::*;

use bytes::Bytes;

#[doc = include_str!("../doc/response-commit.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Commit {
    /// The Merkle root hash of the application state
    ///
    /// XXX(hdevalence) - is this different from an app hash?
    /// XXX(hdevalence) - rename to app_hash ?
    pub data: Bytes,
    /// Blocks below this height may be removed.
    pub retain_height: i64,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::TryFrom;
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<Commit> for pb::ResponseCommit {
    fn from(commit: Commit) -> Self {
        Self {
            data: commit.data,
            retain_height: commit.retain_height,
        }
    }
}

impl TryFrom<pb::ResponseCommit> for Commit {
    type Error = crate::Error;

    fn try_from(commit: pb::ResponseCommit) -> Result<Self, Self::Error> {
        Ok(Self {
            data: commit.data,
            retain_height: commit.retain_height,
        })
    }
}

impl Protobuf<pb::ResponseCommit> for Commit {}
