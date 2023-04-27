use crate::{block, prelude::*};

#[doc = include_str!("../doc/response-commit.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Commit {
    // FIXME: do we want to support this for older versions of the protocol?
    // Does anybody use it?
    //pub data: Bytes,
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
                data: Default::default(),
                retain_height: commit.retain_height.into(),
            }
        }
    }

    impl TryFrom<pb::abci::ResponseCommit> for Commit {
        type Error = crate::Error;

        fn try_from(commit: pb::abci::ResponseCommit) -> Result<Self, Self::Error> {
            Ok(Self {
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
                data: Default::default(),
                retain_height: commit.retain_height.into(),
            }
        }
    }

    impl TryFrom<pb::abci::ResponseCommit> for Commit {
        type Error = crate::Error;

        fn try_from(commit: pb::abci::ResponseCommit) -> Result<Self, Self::Error> {
            Ok(Self {
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
            })
        }
    }

    impl Protobuf<pb::abci::ResponseCommit> for Commit {}
}
