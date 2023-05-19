use crate::{block, Hash};

#[doc = include_str!("../doc/request-extendvote.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExtendVote {
    pub hash: Hash,
    pub height: block::Height,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

mod v0_38 {
    use super::ExtendVote;
    use tendermint_proto::v0_38 as pb;
    use tendermint_proto::Protobuf;

    impl From<ExtendVote> for pb::abci::RequestExtendVote {
        fn from(extend_vote: ExtendVote) -> Self {
            Self {
                hash: extend_vote.hash.into(),
                height: extend_vote.height.into(),
            }
        }
    }

    impl TryFrom<pb::abci::RequestExtendVote> for ExtendVote {
        type Error = crate::Error;

        fn try_from(message: pb::abci::RequestExtendVote) -> Result<Self, Self::Error> {
            Ok(Self {
                hash: message.hash.try_into()?,
                height: message.height.try_into()?,
            })
        }
    }

    impl Protobuf<pb::abci::RequestExtendVote> for ExtendVote {}
}
