use super::Proposal;
use crate::chain::Id as ChainId;
use crate::error::Error;
use crate::prelude::*;
use bytes::BufMut;
use core::convert::{TryFrom, TryInto};
use tendermint_proto::privval::RemoteSignerError;
use tendermint_proto::privval::SignProposalRequest as RawSignProposalRequest;
use tendermint_proto::privval::SignedProposalResponse as RawSignedProposalResponse;
use tendermint_proto::Error as ProtobufError;
use tendermint_proto::Protobuf;

/// SignProposalRequest is a request to sign a proposal
#[derive(Clone, PartialEq, Debug)]
pub struct SignProposalRequest {
    /// Proposal
    pub proposal: Proposal,
    /// Chain ID
    pub chain_id: ChainId,
}

impl Protobuf<RawSignProposalRequest> for SignProposalRequest {}
impl Protobuf<RawSignedProposalResponse> for SignedProposalResponse {}

impl TryFrom<RawSignProposalRequest> for SignProposalRequest {
    type Error = Error;

    fn try_from(value: RawSignProposalRequest) -> Result<Self, Self::Error> {
        if value.proposal.is_none() {
            return Err(Error::no_proposal_found());
        }
        Ok(SignProposalRequest {
            proposal: Proposal::try_from(value.proposal.unwrap())?,
            chain_id: ChainId::try_from(value.chain_id).unwrap(),
        })
    }
}

impl From<SignProposalRequest> for RawSignProposalRequest {
    fn from(value: SignProposalRequest) -> Self {
        RawSignProposalRequest {
            proposal: Some(value.proposal.into()),
            chain_id: value.chain_id.to_string(),
        }
    }
}

impl SignProposalRequest {
    /// Create signable bytes from Proposal.
    pub fn to_signable_bytes<B>(&self, sign_bytes: &mut B) -> Result<bool, ProtobufError>
    where
        B: BufMut,
    {
        self.proposal
            .to_signable_bytes(self.chain_id.clone(), sign_bytes)
    }

    /// Create signable vector from Proposal.
    pub fn to_signable_vec(&self) -> Result<Vec<u8>, ProtobufError> {
        self.proposal.to_signable_vec(self.chain_id.clone())
    }
}

/// SignedProposalResponse is response containing a signed proposal or an error
#[derive(Clone, PartialEq, Debug)]
pub struct SignedProposalResponse {
    /// Proposal
    pub proposal: Option<Proposal>,
    /// Response error
    pub error: Option<RemoteSignerError>,
}

impl TryFrom<RawSignedProposalResponse> for SignedProposalResponse {
    type Error = Error;

    fn try_from(value: RawSignedProposalResponse) -> Result<Self, Self::Error> {
        Ok(SignedProposalResponse {
            proposal: value.proposal.map(TryInto::try_into).transpose()?,
            error: value.error,
        })
    }
}

impl From<SignedProposalResponse> for RawSignedProposalResponse {
    fn from(value: SignedProposalResponse) -> Self {
        RawSignedProposalResponse {
            proposal: value.proposal.map(Into::into),
            error: value.error,
        }
    }
}
