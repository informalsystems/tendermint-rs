use crate::{block, prelude::*};
use bytes::Bytes;

#[doc = include_str!("../doc/response-commit.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Commit {
    /// The Merkle root hash of the application state.
    ///
    /// This field is ignored since CometBFT 0.38.
    pub data: Bytes,
    /// Blocks below this height may be removed.
    pub retain_height: block::Height,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

mod v0_34 {
    use super::Commit;
    use tendermint_proto::v0_34 as pb;
    use tendermint_proto::Protobuf;

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

mod v0_37 {
    use super::Commit;
    use tendermint_proto::v0_37 as pb;
    use tendermint_proto::Protobuf;

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

mod v0_38 {
    use super::Commit;
    use tendermint_proto::v0_38 as pb;
    use tendermint_proto::Protobuf;

    impl From<Commit> for pb::abci::ResponseCommit {
        fn from(commit: Commit) -> Self {
            Self {
                retain_height: commit.retain_height.into(),
            }
        }
    }

    impl TryFrom<pb::abci::ResponseCommit> for Commit {
        type Error = crate::Error;

        fn try_from(commit: pb::abci::ResponseCommit) -> Result<Self, Self::Error> {
            Ok(Self {
                retain_height: commit.retain_height.try_into()?,
                data: Default::default(),
            })
        }
    }

    impl Protobuf<pb::abci::ResponseCommit> for Commit {}
}
