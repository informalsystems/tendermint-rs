use crate::prelude::*;

use bytes::Bytes;

/// XXX(hdevalence): hide merkle::proof and re-export its contents from merkle?
use crate::merkle::proof as merkle;

#[doc = include_str!("../doc/response-query.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Query {
    /// The response code for the query.
    pub code: u32,
    /// The output of the application's logger.
    ///
    /// **May be non-deterministic**.
    pub log: String,
    /// Additional information.
    ///
    /// **May be non-deterministic**.
    pub info: String,
    /// The index of the key in the tree.
    pub index: i64,
    /// The key of the matching data.
    pub key: Bytes,
    /// The value of the matching data.
    pub value: Bytes,
    /// Serialized proof for the value data, if requested, to be verified against
    /// the app hash for the given `height`.
    pub proof: Option<merkle::Proof>,
    /// The block height from which data was derived.
    ///
    /// Note that this is the height of the block containing the application's
    /// Merkle root hash, which represents the state as it was after committing
    /// the block at `height - 1`.
    pub height: i64,
    /// The namespace for the `code`.
    pub codespace: String,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::{TryFrom, TryInto};
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<Query> for pb::ResponseQuery {
    fn from(query: Query) -> Self {
        Self {
            code: query.code,
            log: query.log,
            info: query.info,
            index: query.index,
            key: query.key,
            value: query.value,
            proof_ops: query.proof.map(Into::into),
            height: query.height,
            codespace: query.codespace,
        }
    }
}

impl TryFrom<pb::ResponseQuery> for Query {
    type Error = crate::Error;

    fn try_from(query: pb::ResponseQuery) -> Result<Self, Self::Error> {
        Ok(Self {
            code: query.code,
            log: query.log,
            info: query.info,
            index: query.index,
            key: query.key,
            value: query.value,
            proof: query.proof_ops.map(TryInto::try_into).transpose()?,
            height: query.height,
            codespace: query.codespace,
        })
    }
}

impl Protobuf<pb::ResponseQuery> for Query {}
