use bytes::Bytes;

use crate::{block, prelude::*};

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
    pub height: block::Height,
    /// Whether to return a Merkle proof with the response, if possible.
    pub prove: bool,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::Query;

    impl From<Query> for pb::abci::RequestQuery {
        fn from(query: Query) -> Self {
            Self {
                data: query.data,
                path: query.path,
                height: query.height.into(),
                prove: query.prove,
            }
        }
    }

    impl TryFrom<pb::abci::RequestQuery> for Query {
        type Error = crate::Error;

        fn try_from(query: pb::abci::RequestQuery) -> Result<Self, Self::Error> {
            Ok(Self {
                data: query.data,
                path: query.path,
                height: query.height.try_into()?,
                prove: query.prove,
            })
        }
    }

    impl Protobuf<pb::abci::RequestQuery> for Query {}
}
