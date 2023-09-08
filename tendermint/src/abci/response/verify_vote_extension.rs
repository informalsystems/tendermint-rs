#[doc = include_str!("../doc/response-verifyvoteextension.md")]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum VerifyVoteExtension {
    Unknown = 0,
    Accept = 1,
    Reject = 2,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

mod v0_38 {
    use super::VerifyVoteExtension;
    use crate::Error;
    use tendermint_proto::v0_38::abci as pb;
    use tendermint_proto::Protobuf;

    impl From<VerifyVoteExtension> for pb::ResponseVerifyVoteExtension {
        fn from(value: VerifyVoteExtension) -> Self {
            Self {
                status: value as i32,
            }
        }
    }

    impl TryFrom<pb::ResponseVerifyVoteExtension> for VerifyVoteExtension {
        type Error = Error;

        fn try_from(message: pb::ResponseVerifyVoteExtension) -> Result<Self, Self::Error> {
            use pb::response_verify_vote_extension::VerifyStatus;

            let status = message
                .status
                .try_into()
                .map_err(|_| Error::unsupported_verify_vote_extension_status())?;

            let value = match status {
                VerifyStatus::Unknown => VerifyVoteExtension::Unknown,
                VerifyStatus::Accept => VerifyVoteExtension::Accept,
                VerifyStatus::Reject => VerifyVoteExtension::Reject,
            };
            Ok(value)
        }
    }

    impl Protobuf<pb::ResponseVerifyVoteExtension> for VerifyVoteExtension {}
}
