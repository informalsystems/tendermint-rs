use bytes::Bytes;

use crate::{block, prelude::*};

#[doc = include_str!("../doc/response-commit.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Commit {
    /// The Merkle root hash of the application state
    ///
    /// XXX(hdevalence) - is this different from an app hash?
    /// XXX(hdevalence) - rename to app_hash ?
    pub data: Bytes,
    /// Blocks below this height may be removed.
    pub retain_height: block::Height,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::Commit;

    impl From<Commit> for pb::abci::ResponseCommit {
        fn from(commit: Commit) -> Self {
            Self {
                data: commit.data,
                retain_height: commit.retain_height.into(),
            }
        }
    }

    impl TryFrom<pb::abci::ResponseCommit> for Commit {
        type Error = crate::Error;

        fn try_from(commit: pb::abci::ResponseCommit) -> Result<Self, Self::Error> {
            Ok(Self {
                data: commit.data,
                retain_height: commit.retain_height.try_into()?,
            })
        }
    }

    impl Protobuf<pb::abci::ResponseCommit> for Commit {}
}
