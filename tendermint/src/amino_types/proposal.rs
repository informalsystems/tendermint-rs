use super::{
    block_id::{BlockId, CanonicalBlockId, CanonicalPartSetHeader},
    signature::SignableMsg,
    validate::{
        self, ConsensusMessage, Kind::InvalidMessageType, Kind::MissingConsensusMessage,
        Kind::NegativeHeight, Kind::NegativePOLRound, Kind::NegativeRound,
    },
};
use crate::{
    block::{self, ParseId},
    chain, consensus, error,
};
use bytes::BufMut;
use prost::{EncodeError, Message};
use prost_types::Timestamp;
use std::convert::{TryFrom, TryInto};
use tendermint_proto::privval::RemoteSignerError;
use tendermint_proto::privval::SignProposalRequest as RawSignProposalRequest;
use tendermint_proto::privval::SignedProposalResponse as RawSignedProposalResponse;
use tendermint_proto::types::CanonicalProposal as RawCanonicalProposal;
use tendermint_proto::types::Proposal as RawProposal;
use tendermint_proto::types::SignedMsgType;
use tendermint_proto::DomainType;

#[derive(Clone, PartialEq, Debug, DomainType)]
#[rawtype(RawProposal)]
pub struct Proposal {
    pub msg_type: u16,
    pub height: u32,
    pub round: u16,
    pub pol_round: Option<u16>,
    pub block_id: Option<BlockId>,
    pub timestamp: Option<Timestamp>,
    pub signature: Vec<u8>,
}

impl Proposal {
    pub fn pol_round_to_i32(&self) -> i32 {
        match &self.pol_round {
            Some(u) => *u as i32,
            None => -1,
        }
    }
}

impl TryFrom<RawProposal> for Proposal {
    type Error = validate::Error;

    fn try_from(value: RawProposal) -> Result<Self, Self::Error> {
        if value.r#type < 0 {
            return Err(InvalidMessageType.into());
        }
        if value.height < 0 {
            return Err(NegativeHeight.into());
        }
        if value.round < 0 {
            return Err(NegativeRound.into());
        }
        if value.pol_round < -1 {
            return Err(NegativePOLRound.into());
        }
        let pol_round = match value.pol_round {
            -1 => None,
            n => Some(n as u16),
        };
        let result = Proposal {
            msg_type: value.r#type as u16,
            height: value.height as u32,
            round: value.round as u16,
            pol_round,
            block_id: match value.block_id {
                None => None,
                Some(raw_block_id) => Some(BlockId::try_from(raw_block_id).unwrap()),
            },
            timestamp: value.timestamp,
            signature: value.signature,
        };
        result.validate_basic().map(|_| result)
    }
}

impl From<Proposal> for RawProposal {
    fn from(value: Proposal) -> Self {
        RawProposal {
            r#type: value.msg_type as i32,
            height: value.height as i64,
            round: value.round as i32,
            pol_round: value.pol_round_to_i32(),
            block_id: match value.block_id {
                None => None,
                Some(block_id) => Some(block_id.into()),
            },
            timestamp: value.timestamp,
            signature: value.signature,
        }
    }
}

// TODO(tony): custom derive proc macro for this e.g. `derive(ParseBlockHeight)`
impl block::ParseHeight for Proposal {
    fn parse_block_height(&self) -> Result<block::Height, error::Error> {
        Ok(block::Height::from(self.height))
    }
}

/// SignProposalRequest is a request to sign a proposal
#[derive(Clone, PartialEq, Debug, DomainType)]
#[rawtype(RawSignProposalRequest)]
pub struct SignProposalRequest {
    pub proposal: Option<Proposal>,
    pub chain_id: String,
}

impl TryFrom<RawSignProposalRequest> for SignProposalRequest {
    type Error = validate::Error;

    fn try_from(value: RawSignProposalRequest) -> Result<Self, Self::Error> {
        Ok(SignProposalRequest {
            proposal: match value.proposal {
                None => None,
                Some(proposal) => Some(Proposal::try_from(proposal)?),
            },
            chain_id: value.chain_id,
        })
    }
}

impl From<SignProposalRequest> for RawSignProposalRequest {
    fn from(value: SignProposalRequest) -> Self {
        RawSignProposalRequest {
            proposal: match value.proposal {
                None => None,
                Some(proposal) => Some(proposal.into()),
            },
            chain_id: value.chain_id,
        }
    }
}

#[derive(Clone, PartialEq, DomainType)]
#[rawtype(RawCanonicalProposal)]
pub struct CanonicalProposal {
    /// type alias for byte
    pub msg_type: u16,
    /// canonicalization requires fixed size encoding here
    pub height: u32,
    /// canonicalization requires fixed size encoding here
    pub round: u32,
    pub pol_round: Option<u32>,
    pub block_id: Option<CanonicalBlockId>,
    pub timestamp: Option<Timestamp>,
    pub chain_id: String,
}

impl CanonicalProposal {
    pub fn pol_round_to_i64(&self) -> i64 {
        match &self.pol_round {
            None => -1,
            Some(u) => *u as i64,
        }
    }
}

impl TryFrom<RawCanonicalProposal> for CanonicalProposal {
    type Error = validate::Error;

    fn try_from(value: RawCanonicalProposal) -> Result<Self, Self::Error> {
        if value.r#type < 0 {
            return Err(InvalidMessageType.into());
        }
        if value.height < 0 {
            return Err(NegativeHeight.into());
        }
        if value.round < 0 {
            return Err(NegativeRound.into());
        }
        if value.pol_round < -1 {
            return Err(NegativePOLRound.into());
        }
        let pol_round = match value.pol_round {
            -1 => None,
            n => Some(n as u32),
        };
        Ok(CanonicalProposal {
            msg_type: value.r#type as u16,
            height: value.height as u32,
            round: value.round as u32,
            pol_round,
            block_id: match value.block_id {
                None => None,
                Some(block_id) => Some(block_id.try_into()?),
            },
            timestamp: value.timestamp,
            chain_id: value.chain_id,
        })
    }
}

impl From<CanonicalProposal> for RawCanonicalProposal {
    fn from(value: CanonicalProposal) -> Self {
        RawCanonicalProposal {
            r#type: value.msg_type as i32,
            height: value.height as i64,
            round: value.round as i64,
            pol_round: value.pol_round_to_i64(),
            block_id: match value.block_id {
                None => None,
                Some(block_id) => Some(block_id.into()),
            },
            timestamp: value.timestamp,
            chain_id: value.chain_id,
        }
    }
}

impl chain::ParseId for CanonicalProposal {
    fn parse_chain_id(&self) -> Result<chain::Id, error::Error> {
        self.chain_id.parse()
    }
}

impl block::ParseHeight for CanonicalProposal {
    fn parse_block_height(&self) -> Result<block::Height, error::Error> {
        Ok(block::Height::from(self.height))
    }
}

/// SignedProposalResponse is response containing a signed proposal or an error
#[derive(Clone, PartialEq, DomainType)]
#[rawtype(RawSignedProposalResponse)]
pub struct SignedProposalResponse {
    pub proposal: Option<Proposal>,
    pub error: Option<RemoteSignerError>,
}

impl TryFrom<RawSignedProposalResponse> for SignedProposalResponse {
    type Error = validate::Error;

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
    fn sign_bytes<B>(&self, chain_id: chain::Id, sign_bytes: &mut B) -> Result<bool, EncodeError>
    where
        B: BufMut,
    {
        let mut spr = self.clone();
        if let Some(ref mut pr) = spr.proposal {
            pr.signature = vec![];
        }
        let proposal = spr.proposal.unwrap();
        let cp = CanonicalProposal {
            chain_id: chain_id.to_string(),
            msg_type: SignedMsgType::Proposal as u16,
            height: proposal.height,
            block_id: match proposal.block_id {
                Some(bid) => Some(CanonicalBlockId {
                    hash: bid.hash,
                    part_set_header: match bid.part_set_header {
                        Some(psh) => Some(CanonicalPartSetHeader {
                            hash: psh.hash,
                            total: psh.total as u32,
                        }),
                        None => None,
                    },
                }),
                None => None,
            },
            pol_round: proposal.pol_round.map(|n| n as u32),
            round: proposal.round as u32,
            timestamp: proposal.timestamp,
        };

        RawCanonicalProposal::from(cp).encode_length_delimited(sign_bytes)?;
        Ok(true)
    }
    fn set_signature(&mut self, sig: &ed25519::Signature) {
        if let Some(ref mut prop) = self.proposal {
            prop.signature = sig.as_ref().to_vec();
        }
    }
    fn validate(&self) -> Result<(), validate::Error> {
        match self.proposal {
            Some(ref p) => p.validate_basic(),
            None => Err(MissingConsensusMessage.into()),
        }
    }
    fn consensus_state(&self) -> Option<consensus::State> {
        match self.proposal {
            Some(ref p) => Some(consensus::State {
                height: block::Height::from(p.height),
                round: p.round as i64,
                step: 3,
                block_id: {
                    match p.block_id {
                        Some(ref b) => match b.parse_block_id() {
                            Ok(id) => Some(id),
                            Err(_) => None,
                        },
                        None => None,
                    }
                },
            }),
            None => None,
        }
    }

    fn height(&self) -> Option<i64> {
        self.proposal
            .as_ref()
            .map(|proposal| proposal.height as i64)
    }

    fn msg_type(&self) -> Option<SignedMsgType> {
        Some(SignedMsgType::Proposal)
    }
}

impl ConsensusMessage for Proposal {
    fn validate_basic(&self) -> Result<(), validate::Error> {
        if self.msg_type != SignedMsgType::Proposal as u16 {
            return Err(InvalidMessageType.into());
        }
        // TODO validate proposal's block_id

        // signature will be missing as the KMS provides it

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amino_types::block_id::{BlockId, PartSetHeader};
    use crate::chain::Id;
    use chrono::{DateTime, Utc};

    #[test]
    fn test_serialization() {
        let dt = "2018-02-11T07:09:22.765Z".parse::<DateTime<Utc>>().unwrap();
        let t = Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        };
        let proposal = Proposal {
            msg_type: SignedMsgType::Proposal as u16,
            height: 12345,
            round: 23456,
            pol_round: None,
            block_id: Some(BlockId {
                hash: b"DEADBEEFDEADBEEFBAFBAFBAFBAFBAFA".to_vec(),
                part_set_header: Some(PartSetHeader {
                    total: 65535,
                    hash: b"0022446688AACCEE1133557799BBDDFF".to_vec(),
                }),
            }),
            timestamp: Some(t),
            signature: vec![],
        };
        let mut got = vec![];

        let request = SignProposalRequest {
            proposal: Some(proposal),
            chain_id: "test_chain_id".to_string(),
        };
        let _have = request.sign_bytes(Id::from("test_chain_id"), &mut got);

        // the following vector is generated via:
        /*
           import (
               "fmt"
               prototypes "github.com/tendermint/tendermint/proto/tendermint/types"
               "github.com/tendermint/tendermint/types"
               "strings"
               "time"
           )
           func proposalSerialize() {
               stamp, _ := time.Parse(time.RFC3339Nano, "2018-02-11T07:09:22.765Z")
               proposal := &types.Proposal{
                   Type:     prototypes.SignedMsgType(prototypes.ProposalType),
                   Height:   12345,
                   Round:    23456,
                   POLRound: -1,
                   BlockID: types.BlockID{
                       Hash: []byte("DEADBEEFDEADBEEFBAFBAFBAFBAFBAFA"),
                       PartSetHeader: types.PartSetHeader{
                           Hash:  []byte("0022446688AACCEE1133557799BBDDFF"),
                           Total: 65535,
                       },
                   },
                   Timestamp: stamp,
               }
               signBytes := types.ProposalSignBytes("test_chain_id",proposal.ToProto())
               fmt.Println(strings.Join(strings.Split(fmt.Sprintf("%v", signBytes), " "), ", "))
           }
        */

        let want = vec![
            136, 1, 8, 32, 17, 57, 48, 0, 0, 0, 0, 0, 0, 25, 160, 91, 0, 0, 0, 0, 0, 0, 32, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 1, 42, 74, 10, 32, 68, 69, 65, 68, 66, 69, 69,
            70, 68, 69, 65, 68, 66, 69, 69, 70, 66, 65, 70, 66, 65, 70, 66, 65, 70, 66, 65, 70, 66,
            65, 70, 65, 18, 38, 8, 255, 255, 3, 18, 32, 48, 48, 50, 50, 52, 52, 54, 54, 56, 56, 65,
            65, 67, 67, 69, 69, 49, 49, 51, 51, 53, 53, 55, 55, 57, 57, 66, 66, 68, 68, 70, 70, 50,
            12, 8, 162, 216, 255, 211, 5, 16, 192, 242, 227, 236, 2, 58, 13, 116, 101, 115, 116,
            95, 99, 104, 97, 105, 110, 95, 105, 100,
        ];

        assert_eq!(got, want)
    }

    #[test]
    fn test_deserialization() {
        let dt = "2018-02-11T07:09:22.765Z".parse::<DateTime<Utc>>().unwrap();
        let t = Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        };
        let proposal = Proposal {
            msg_type: SignedMsgType::Proposal as u16,
            height: 12345,
            round: 23456,
            timestamp: Some(t),

            pol_round: None,
            block_id: Some(BlockId {
                hash: b"DEADBEEFDEADBEEFBAFBAFBAFBAFBAFA".to_vec(),
                part_set_header: Some(PartSetHeader {
                    total: 65535,
                    hash: b"0022446688AACCEE1133557799BBDDFF".to_vec(),
                }),
            }),
            signature: vec![],
        };
        let want = SignProposalRequest {
            proposal: Some(proposal),
            chain_id: "test_chain_id".to_string(),
        };

        let data = vec![
            10, 110, 8, 32, 16, 185, 96, 24, 160, 183, 1, 32, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 1, 42, 74, 10, 32, 68, 69, 65, 68, 66, 69, 69, 70, 68, 69, 65, 68, 66, 69,
            69, 70, 66, 65, 70, 66, 65, 70, 66, 65, 70, 66, 65, 70, 66, 65, 70, 65, 18, 38, 8, 255,
            255, 3, 18, 32, 48, 48, 50, 50, 52, 52, 54, 54, 56, 56, 65, 65, 67, 67, 69, 69, 49, 49,
            51, 51, 53, 53, 55, 55, 57, 57, 66, 66, 68, 68, 70, 70, 50, 12, 8, 162, 216, 255, 211,
            5, 16, 192, 242, 227, 236, 2, 18, 13, 116, 101, 115, 116, 95, 99, 104, 97, 105, 110,
            95, 105, 100,
        ];

        match SignProposalRequest::decode(data.as_ref()) {
            Ok(have) => assert_eq!(have, want),
            Err(err) => panic!(err.to_string()),
        }
    }
}
