use crate::prelude::*;

#[doc = include_str!("../doc/response-processproposal.md")]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(i32)]
#[derive(Default)]
pub enum ProcessProposal {
    #[default]
    Unknown = 0,
    Accept = 1,
    Reject = 2,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

mod v0_37 {
    use super::ProcessProposal;
    use crate::Error;
    use tendermint_proto::v0_37::abci as pb;
    use tendermint_proto::Protobuf;

    impl From<ProcessProposal> for pb::ResponseProcessProposal {
        fn from(value: ProcessProposal) -> Self {
            Self {
                status: value as i32,
            }
        }
    }

    impl TryFrom<pb::ResponseProcessProposal> for ProcessProposal {
        type Error = Error;

        fn try_from(message: pb::ResponseProcessProposal) -> Result<Self, Self::Error> {
            use pb::response_process_proposal::ProposalStatus;

            let status = message
                .status
                .try_into()
                .map_err(|_| Error::unsupported_process_proposal_status())?;

            let value = match status {
                ProposalStatus::Unknown => ProcessProposal::Unknown,
                ProposalStatus::Accept => ProcessProposal::Accept,
                ProposalStatus::Reject => ProcessProposal::Reject,
            };
            Ok(value)
        }
    }

    impl Protobuf<pb::ResponseProcessProposal> for ProcessProposal {}
}

mod v0_38 {
    use super::ProcessProposal;
    use crate::Error;
    use tendermint_proto::v0_38::abci as pb;
    use tendermint_proto::Protobuf;

    impl From<ProcessProposal> for pb::ResponseProcessProposal {
        fn from(value: ProcessProposal) -> Self {
            Self {
                status: value as i32,
            }
        }
    }

    impl TryFrom<pb::ResponseProcessProposal> for ProcessProposal {
        type Error = Error;

        fn try_from(message: pb::ResponseProcessProposal) -> Result<Self, Self::Error> {
            use pb::response_process_proposal::ProposalStatus;

            let status = message
                .status
                .try_into()
                .map_err(|_| Error::unsupported_process_proposal_status())?;

            let value = match status {
                ProposalStatus::Unknown => ProcessProposal::Unknown,
                ProposalStatus::Accept => ProcessProposal::Accept,
                ProposalStatus::Reject => ProcessProposal::Reject,
            };
            Ok(value)
        }
    }

    impl Protobuf<pb::ResponseProcessProposal> for ProcessProposal {}
}
