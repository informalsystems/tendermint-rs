use super::{
    block_id::{BlockId, CanonicalBlockId, CanonicalPartSetHeader},
    signature::{SignableMsg, SignedMsgType},
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
use std::convert::TryFrom;
use tendermint_proto::privval::RemoteSignerError;

// Copied from tendermint_proto::types::Proposal
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Proposal {
    #[prost(enumeration = "SignedMsgType", tag = "1")]
    pub r#type: i32,
    #[prost(int64, tag = "2")]
    pub height: i64,
    #[prost(int32, tag = "3")]
    pub round: i32,
    #[prost(int32, tag = "4")]
    pub pol_round: i32,
    #[prost(message, optional, tag = "5")]
    pub block_id: ::std::option::Option<BlockId>,
    #[prost(message, optional, tag = "6")]
    pub timestamp: ::std::option::Option<::prost_types::Timestamp>,
    #[prost(bytes, tag = "7")]
    pub signature: Vec<u8>,
}

// TODO(tony): custom derive proc macro for this e.g. `derive(ParseBlockHeight)`
impl block::ParseHeight for Proposal {
    fn parse_block_height(&self) -> Result<block::Height, error::Error> {
        block::Height::try_from(self.height)
    }
}

// Copied from tendermint_proto::privval::SignProposalRequest
/// SignProposalRequest is a request to sign a proposal
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignProposalRequest {
    #[prost(message, optional, tag = "1")]
    pub proposal: ::std::option::Option<Proposal>,
    #[prost(string, tag = "2")]
    pub chain_id: String,
}

// Copied from tendermint_proto::types::CanonicalProposal
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CanonicalProposal {
    /// type alias for byte
    #[prost(enumeration = "SignedMsgType", tag = "1")]
    pub r#type: i32,
    /// canonicalization requires fixed size encoding here
    #[prost(sfixed64, tag = "2")]
    pub height: i64,
    /// canonicalization requires fixed size encoding here
    #[prost(sfixed64, tag = "3")]
    pub round: i64,
    #[prost(int64, tag = "4")]
    pub pol_round: i64,
    #[prost(message, optional, tag = "5")]
    pub block_id: ::std::option::Option<CanonicalBlockId>,
    #[prost(message, optional, tag = "6")]
    pub timestamp: ::std::option::Option<::prost_types::Timestamp>,
    #[prost(string, tag = "7")]
    pub chain_id: String,
}

impl chain::ParseId for CanonicalProposal {
    fn parse_chain_id(&self) -> Result<chain::Id, error::Error> {
        self.chain_id.parse()
    }
}

impl block::ParseHeight for CanonicalProposal {
    fn parse_block_height(&self) -> Result<block::Height, error::Error> {
        block::Height::try_from(self.height)
    }
}

// Copied from tendermint_proto::privval::SignedProposalResponse
/// SignedProposalResponse is response containing a signed proposal or an error
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedProposalResponse {
    #[prost(message, optional, tag = "1")]
    pub proposal: ::std::option::Option<Proposal>,
    #[prost(message, optional, tag = "2")]
    pub error: ::std::option::Option<RemoteSignerError>,
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
            r#type: SignedMsgType::Proposal as i32,
            height: proposal.height,
            block_id: match proposal.block_id {
                Some(bid) => Some(CanonicalBlockId {
                    hash: bid.hash,
                    part_set_header: match bid.part_set_header {
                        Some(psh) => Some(CanonicalPartSetHeader {
                            hash: psh.hash,
                            total: psh.total,
                        }),
                        None => None,
                    },
                }),
                None => None,
            },
            pol_round: proposal.pol_round as i64,
            round: proposal.round as i64,
            timestamp: proposal.timestamp,
        };

        cp.encode_length_delimited(sign_bytes)?;
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
                height: match block::Height::try_from(p.height) {
                    Ok(h) => h,
                    Err(_err) => return None, // TODO(tarcieri): return an error?
                },
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
        self.proposal.as_ref().map(|proposal| proposal.height)
    }

    fn msg_type(&self) -> Option<SignedMsgType> {
        Some(SignedMsgType::Proposal)
    }
}

impl ConsensusMessage for Proposal {
    fn validate_basic(&self) -> Result<(), validate::Error> {
        if self.r#type != SignedMsgType::Proposal {
            return Err(InvalidMessageType.into());
        }
        if self.height < 0 {
            return Err(NegativeHeight.into());
        }
        if self.round < 0 {
            return Err(NegativeRound.into());
        }
        if self.pol_round < -1 {
            return Err(NegativePOLRound.into());
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
    use chrono::{DateTime, Utc};
    use prost::Message;
    use prost_types::Timestamp;
    use crate::chain::Id;

    #[test]
    fn test_serialization() {
        let dt = "2018-02-11T07:09:22.765Z".parse::<DateTime<Utc>>().unwrap();
        let t = Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        };
        let proposal = Proposal {
            r#type: SignedMsgType::Proposal as i32,
            height: 12345,
            round: 23456,
            pol_round: -1,
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

        let request = SignProposalRequest{ proposal: Some(proposal), chain_id: "test_chain_id".to_string() };
        let _have = request.sign_bytes(Id::from("test_chain_id"),&mut got);

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
            r#type: SignedMsgType::Proposal as i32,
            height: 12345,
            round: 23456,
            timestamp: Some(t),

            pol_round: -1,
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
