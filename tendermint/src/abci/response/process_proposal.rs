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

// ProcessProposal has been introduced in 0.37.

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
    type Error = crate::Error;

    fn try_from(message: pb::ResponseProcessProposal) -> Result<Self, Self::Error> {
        let value = match message.status {
            0 => ProcessProposal::Unknown,
            1 => ProcessProposal::Accept,
            2 => ProcessProposal::Reject,
            _ => return Err(crate::Error::unsupported_process_proposal_status()),
        };
        Ok(value)
    }
}

impl Protobuf<pb::ResponseProcessProposal> for ProcessProposal {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorDetail;

    use std::collections::HashMap;

    #[test]
    fn all_status_values_are_covered() {
        use pb::response_process_proposal::ProposalStatus::*;

        const FIRST_INVALID_STATUS: i32 = 3;

        let mut covered = HashMap::new();
        for v in [Unknown, Accept, Reject] {
            // Match the generated enum values exhaustively
            match v {
                Unknown | Accept | Reject => {
                    covered.insert(v as i32, false);
                },
            }
        }
        for status in 0..FIRST_INVALID_STATUS {
            let message = pb::ResponseProcessProposal { status };
            let response: ProcessProposal = message.try_into().unwrap();
            assert_eq!(response as i32, status);
            covered.insert(status, true);
        }
        assert!(covered.values().all(|&x| x));

        let message = pb::ResponseProcessProposal {
            status: FIRST_INVALID_STATUS,
        };
        let err = ProcessProposal::try_from(message).err().unwrap();
        assert!(matches!(
            err.0,
            ErrorDetail::UnsupportedProcessProposalStatus(_),
        ));
    }
}
