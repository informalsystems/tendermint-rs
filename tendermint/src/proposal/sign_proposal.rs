use super::{Proposal, CanonicalProposal, Type};
use crate::Error;
use tendermint_proto::{DomainType, Error as DomainTypeError};
use std::convert::TryFrom;
use tendermint_proto::privval::RemoteSignerError;
use tendermint_proto::privval::SignProposalRequest as RawSignProposalRequest;
use tendermint_proto::privval::SignedProposalResponse as RawSignedProposalResponse;
use crate::signature::SignableMsg;
use crate::chain::{Id as ChainId, Id};
use crate::block::Id as BlockId;
use crate::block::parts::Header;
use std::str::FromStr;
use bytes::BufMut;
use crate::signature::Signature::Ed25519;
use ed25519::{Signature as Ed25519Signature, SIGNATURE_LENGTH};
use crate::Signature;
use crate::consensus::State;

/// SignProposalRequest is a request to sign a proposal
#[derive(Clone, PartialEq, Debug)]
pub struct SignProposalRequest {
    /// Proposal
    pub proposal: Option<Proposal>,
    /// Chain ID
    pub chain_id: ChainId,
}

impl DomainType<RawSignProposalRequest> for SignProposalRequest {}
impl DomainType<RawSignedProposalResponse> for SignedProposalResponse {}

impl TryFrom<RawSignProposalRequest> for SignProposalRequest {
    type Error = Error;

    fn try_from(value: RawSignProposalRequest) -> Result<Self, Self::Error> {
        Ok(SignProposalRequest {
            proposal: match value.proposal {
                None => None,
                Some(proposal) => Some(Proposal::try_from(proposal)?),
            },
            chain_id: ChainId::from_str(value.chain_id.as_str()).unwrap(),
        })
    }
}

impl From<SignProposalRequest> for RawSignProposalRequest {
    fn from(value: SignProposalRequest) -> Self {
        RawSignProposalRequest {
            proposal: value.proposal.map(|p| p.into()),
            chain_id: value.chain_id.as_str().to_string(),
        }
    }
}

/// SignedProposalResponse is response containing a signed proposal or an error
#[derive(Clone, PartialEq)]
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
            proposal: match value.proposal {
                None => None,
                Some(proposal) => Some(Proposal::try_from(proposal)?),
            },
            error: value.error,
        })
    }
}

impl From<SignedProposalResponse> for RawSignedProposalResponse {
    fn from(value: SignedProposalResponse) -> Self {
        RawSignedProposalResponse {
            proposal: match value.proposal {
                None => None,
                Some(proposal) => Some(proposal.into()),
            },
            error: value.error,
        }
    }
}

impl SignableMsg for SignProposalRequest {
    fn sign_bytes<B>(&self, chain_id: ChainId, sign_bytes: &mut B) -> Result<bool, Error>
        where
            B: BufMut,
    {
        let mut spr = self.clone();
        if let Some(ref mut pr) = spr.proposal {
            pr.signature = Ed25519(Ed25519Signature::new([0; SIGNATURE_LENGTH]));
        }
        let proposal = spr.proposal.unwrap();
        let cp = CanonicalProposal {
            chain_id,
            msg_type: Type::Proposal,
            height: proposal.height,
            block_id: match proposal.block_id {
                Some(bid) => Some(BlockId {
                    hash: bid.hash,
                    parts: match bid.parts {
                        Some(psh) => Some(Header {
                            hash: psh.hash,
                            total: psh.total,
                        }),
                        None => None,
                    },
                }),
                None => None,
            },
            pol_round: proposal.pol_round,
            round: proposal.round,
            timestamp: proposal.timestamp,
        };

        cp.encode_length_delimited(sign_bytes)?;
        Ok(true)
    }
    fn sign_vec(&self, chain_id: Id) -> Result<Vec<u8>, DomainTypeError> {
        CanonicalProposal::new(self.proposal.clone().unwrap(), chain_id).encode_length_delimited_vec()
    }
    fn set_signature(&mut self, sig: Signature) {
        if let Some(ref mut prop) = self.proposal {
            prop.signature = sig;
        }
    }
    fn consensus_state(&self) -> Option<State> {
        match self.proposal {
            Some(ref p) => Some(State {
                height: p.height,
                round: p.round,
                step: 3,
                block_id: p.block_id.clone(),
            }),
            None => None,
        }
    }
}
