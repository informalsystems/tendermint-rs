use bytes::Bytes;

#[doc = include_str!("../doc/response-extendvote.md")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtendVote {
    pub vote_extension: Bytes,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

mod v0_38 {
    use super::ExtendVote;
    use tendermint_proto::v0_38::abci as pb;
    use tendermint_proto::Protobuf;

    impl From<ExtendVote> for pb::ResponseExtendVote {
        fn from(value: ExtendVote) -> Self {
            Self {
                vote_extension: value.vote_extension,
            }
        }
    }

    impl TryFrom<pb::ResponseExtendVote> for ExtendVote {
        type Error = crate::Error;

        fn try_from(message: pb::ResponseExtendVote) -> Result<Self, Self::Error> {
            Ok(Self {
                vote_extension: message.vote_extension,
            })
        }
    }

    impl Protobuf<pb::ResponseExtendVote> for ExtendVote {}
}
