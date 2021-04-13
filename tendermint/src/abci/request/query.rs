use crate::prelude::*;

use bytes::Bytes;

#[doc = include_str!("../doc/request-query.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Query {
    /// Raw query bytes.
    ///
    /// Can be used with or in lieu of `path`.
    pub data: Bytes,
    /// Path of the request, like an HTTP `GET` path.
    ///
    /// Can be used with or in lieu of `data`.
    ///
    /// Applications MUST interpret `/store` as a query by key on the underlying
    /// store. The key SHOULD be specified in the Data field. Applications SHOULD
    /// allow queries over specific types like `/accounts/...` or `/votes/...`.
    pub path: String,
    /// The block height for which the query should be executed.
    ///
    /// The default `0` returns data for the latest committed block. Note that
    /// this is the height of the block containing the application's Merkle root
    /// hash, which represents the state as it was after committing the block at
    /// `height - 1`.
    pub height: i64,
    /// Whether to return a Merkle proof with the response, if possible.
    pub prove: bool,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::TryFrom;
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<Query> for pb::RequestQuery {
    fn from(query: Query) -> Self {
        Self {
            data: query.data,
            path: query.path,
            height: query.height,
            prove: query.prove,
        }
    }
}

impl TryFrom<pb::RequestQuery> for Query {
    type Error = crate::Error;

    fn try_from(query: pb::RequestQuery) -> Result<Self, Self::Error> {
        Ok(Self {
            data: query.data,
            path: query.path,
            height: query.height,
            prove: query.prove,
        })
    }
}

impl Protobuf<pb::RequestQuery> for Query {}
