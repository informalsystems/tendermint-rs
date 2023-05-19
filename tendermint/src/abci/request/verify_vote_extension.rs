use crate::{account, block, Hash};
use bytes::Bytes;

#[doc = include_str!("../doc/request-verifyvoteextension.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VerifyVoteExtension {
    pub hash: Hash,
    pub validator_address: account::Id,
    pub height: block::Height,
    pub vote_extension: Bytes,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

mod v0_38 {
    use super::VerifyVoteExtension;
    use tendermint_proto::v0_38 as pb;
    use tendermint_proto::Protobuf;

    impl From<VerifyVoteExtension> for pb::abci::RequestVerifyVoteExtension {
        fn from(value: VerifyVoteExtension) -> Self {
            Self {
                hash: value.hash.into(),
                validator_address: value.validator_address.into(),
                height: value.height.into(),
                vote_extension: value.vote_extension,
            }
        }
    }

    impl TryFrom<pb::abci::RequestVerifyVoteExtension> for VerifyVoteExtension {
        type Error = crate::Error;

        fn try_from(message: pb::abci::RequestVerifyVoteExtension) -> Result<Self, Self::Error> {
            Ok(Self {
                hash: message.hash.try_into()?,
                validator_address: message.validator_address.try_into()?,
                height: message.height.try_into()?,
                vote_extension: message.vote_extension,
            })
        }
    }

    impl Protobuf<pb::abci::RequestVerifyVoteExtension> for VerifyVoteExtension {}
}
