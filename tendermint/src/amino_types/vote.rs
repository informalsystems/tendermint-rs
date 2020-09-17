use super::{
    block_id::{BlockId, CanonicalBlockId, CanonicalPartSetHeader},
    signature::SignableMsg,
    validate,
    validate::{ConsensusMessage, Kind::*},
};
use crate::amino_types::PartSetHeader;
use crate::{
    block::{self, ParseId},
    chain, consensus,
    error::Error,
    vote,
};
use bytes::BufMut;
use prost::EncodeError;
use prost_types::Timestamp;
use std::convert::{TryFrom, TryInto};
use tendermint_proto::privval::SignedVoteResponse as RawSignedVoteResponse;
use tendermint_proto::privval::{RemoteSignerError, SignVoteRequest as RawSignVoteRequest};
use tendermint_proto::types::CanonicalVote as RawCanonicalVote;
use tendermint_proto::types::SignedMsgType;
use tendermint_proto::types::Vote as RawVote;
use tendermint_proto::DomainType;

const VALIDATOR_ADDR_SIZE: usize = 20;

/// Vote represents a prevote, precommit, or commit vote from validators for consensus.
#[derive(Clone, PartialEq, Default, Debug, DomainType)]
#[rawtype(RawVote)]
pub struct Vote {
    pub vote_type: u16,
    pub height: u32,
    pub round: u16,
    /// zero if vote is nil.
    pub block_id: ::std::option::Option<BlockId>,
    pub timestamp: ::std::option::Option<::prost_types::Timestamp>,
    pub validator_address: Vec<u8>,
    pub validator_index: u16,
    pub signature: Vec<u8>,
}

impl TryFrom<RawVote> for Vote {
    type Error = validate::Error;

    fn try_from(value: RawVote) -> Result<Self, Self::Error> {
        if value.r#type < 0 {
            return Err(InvalidMessageType.into());
        }
        if value.height < 0 {
            return Err(NegativeHeight.into());
        }
        if value.round < 0 {
            return Err(NegativeRound.into());
        }
        if value.validator_index < 0 {
            return Err(NegativeValidatorIndex.into());
        }
        Ok(Vote {
            vote_type: value.r#type as u16,
            height: value.height as u32,
            round: value.round as u16,
            block_id: value.block_id.map(|f| BlockId::try_from(f).unwrap()),
            timestamp: value.timestamp,
            validator_address: value.validator_address,
            validator_index: value.validator_index as u16,
            signature: value.signature,
        })
    }
}

impl From<Vote> for RawVote {
    fn from(value: Vote) -> Self {
        RawVote {
            r#type: value.vote_type as i32,
            height: value.height as i64,
            round: value.round as i32,
            block_id: value.block_id.map(|b| b.into()),
            timestamp: value.timestamp,
            validator_address: value.validator_address,
            validator_index: value.validator_index as i32,
            signature: value.signature,
        }
    }
}

impl Vote {
    fn msg_type(&self) -> Option<SignedMsgType> {
        if self.vote_type == SignedMsgType::Prevote as u16 {
            Some(SignedMsgType::Prevote)
        } else if self.vote_type == SignedMsgType::Precommit as u16 {
            Some(SignedMsgType::Precommit)
        } else {
            None
        }
    }
}

impl From<&vote::Vote> for Vote {
    fn from(vote: &vote::Vote) -> Self {
        Vote {
            vote_type: vote.vote_type as u16,
            height: vote.height.value() as u32, // TODO potential overflow :-/
            round: vote.round as u16,           // TODO potential overflow :-/
            block_id: vote.block_id.as_ref().map(|block_id| BlockId {
                hash: block_id.hash.as_bytes().to_vec(),
                part_set_header: block_id.parts.as_ref().map(PartSetHeader::from),
            }),
            timestamp: Some(Timestamp::from(vote.timestamp.to_system_time().unwrap())),
            validator_address: vote.validator_address.as_bytes().to_vec(),
            validator_index: vote.validator_index,
            signature: vote.signature.as_bytes().to_vec(),
        }
    }
}

impl block::ParseHeight for Vote {
    fn parse_block_height(&self) -> Result<block::Height, Error> {
        Ok(block::Height::from(self.height as u64))
    }
}

/// SignVoteRequest is a request to sign a vote
#[derive(Clone, PartialEq, Debug, DomainType)]
#[rawtype(RawSignVoteRequest)]
pub struct SignVoteRequest {
    pub vote: Option<Vote>,
    pub chain_id: String,
}

impl TryFrom<RawSignVoteRequest> for SignVoteRequest {
    type Error = validate::Error;

    fn try_from(value: RawSignVoteRequest) -> Result<Self, Self::Error> {
        Ok(SignVoteRequest {
            vote: match value.vote {
                None => None,
                Some(vote) => Some(Vote::try_from(vote)?),
            },
            chain_id: value.chain_id,
        })
    }
}

impl From<SignVoteRequest> for RawSignVoteRequest {
    fn from(value: SignVoteRequest) -> Self {
        RawSignVoteRequest {
            vote: match value.vote {
                None => None,
                Some(vote) => Some(vote.into()),
            },
            chain_id: value.chain_id,
        }
    }
}

/// SignedVoteResponse is a response containing a signed vote or an error
#[derive(Clone, PartialEq)]
pub struct SignedVoteResponse {
    pub vote: Option<Vote>,
    pub error: Option<RemoteSignerError>,
}

impl TryFrom<RawSignedVoteResponse> for SignedVoteResponse {
    type Error = validate::Error;

    fn try_from(value: RawSignedVoteResponse) -> Result<Self, Self::Error> {
        Ok(SignedVoteResponse {
            vote: match value.vote {
                None => None,
                Some(vote) => Some(Vote::try_from(vote)?),
            },
            error: value.error,
        })
    }
}

impl TryFrom<SignedVoteResponse> for RawSignedVoteResponse {
    type Error = validate::Error;

    fn try_from(value: SignedVoteResponse) -> Result<Self, Self::Error> {
        Ok(RawSignedVoteResponse {
            vote: match value.vote {
                None => None,
                Some(vote) => Some(RawVote::try_from(vote)?),
            },
            error: value.error,
        })
    }
}

#[derive(Clone, PartialEq, DomainType)]
#[rawtype(RawCanonicalVote)]
pub struct CanonicalVote {
    pub vote_type: u16,
    pub height: i64,
    pub round: i64,
    pub block_id: Option<CanonicalBlockId>,
    pub timestamp: Option<Timestamp>,
    pub chain_id: String,
}

impl TryFrom<RawCanonicalVote> for CanonicalVote {
    type Error = validate::Error;

    fn try_from(value: RawCanonicalVote) -> Result<Self, Self::Error> {
        if value.r#type < 0 {
            return Err(InvalidMessageType.into());
        }
        Ok(CanonicalVote {
            vote_type: value.r#type as u16,
            height: value.height,
            round: value.round,
            block_id: value.block_id.map(|r| r.try_into().unwrap()),
            timestamp: value.timestamp,
            chain_id: value.chain_id,
        })
    }
}

impl From<CanonicalVote> for RawCanonicalVote {
    fn from(value: CanonicalVote) -> Self {
        RawCanonicalVote {
            r#type: value.vote_type as i32,
            height: value.height,
            round: value.round,
            block_id: value.block_id.map(|b| b.into()),
            timestamp: value.timestamp,
            chain_id: value.chain_id,
        }
    }
}

impl chain::ParseId for CanonicalVote {
    fn parse_chain_id(&self) -> Result<chain::Id, Error> {
        self.chain_id.parse()
    }
}

impl block::ParseHeight for CanonicalVote {
    fn parse_block_height(&self) -> Result<block::Height, Error> {
        block::Height::try_from(self.height)
    }
}

impl CanonicalVote {
    pub fn new(vote: Vote, chain_id: &str) -> CanonicalVote {
        CanonicalVote {
            vote_type: vote.vote_type,
            chain_id: chain_id.to_string(),
            block_id: match vote.block_id {
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
            height: vote.height as i64,
            round: vote.round as i64,
            timestamp: match vote.timestamp {
                None => Some(Timestamp {
                    seconds: -62_135_596_800,
                    nanos: 0,
                }),
                Some(t) => Some(t),
            },
        }
    }
}

impl SignableMsg for SignVoteRequest {
    fn sign_bytes<B>(&self, chain_id: chain::Id, sign_bytes: &mut B) -> Result<bool, EncodeError>
    where
        B: BufMut,
    {
        let mut svr = self.clone();
        if let Some(ref mut vo) = svr.vote {
            vo.signature = vec![];
        }
        let vote = svr.vote.unwrap();
        let cv = CanonicalVote::new(vote, chain_id.as_str());

        cv.encode_length_delimited(sign_bytes).unwrap(); // Todo: Handle the single "EncodeError"
        Ok(true)
    }
    fn set_signature(&mut self, sig: &ed25519::Signature) {
        if let Some(ref mut vt) = self.vote {
            vt.signature = sig.as_ref().to_vec();
        }
    }
    fn validate(&self) -> Result<(), validate::Error> {
        match self.vote {
            Some(ref v) => v.validate_basic(),
            None => Err(MissingConsensusMessage.into()),
        }
    }
    fn consensus_state(&self) -> Option<consensus::State> {
        match self.vote {
            Some(ref v) => Some(consensus::State {
                height: block::Height::from(v.height as u64),
                round: v.round as i64,
                step: 6,
                block_id: {
                    match v.block_id {
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
        self.vote.as_ref().map(|vote| vote.height as i64)
    }
    fn msg_type(&self) -> Option<SignedMsgType> {
        self.vote.as_ref().and_then(|vote| vote.msg_type())
    }
}

impl ConsensusMessage for Vote {
    fn validate_basic(&self) -> Result<(), validate::Error> {
        if self.msg_type().is_none() {
            return Err(InvalidMessageType.into());
        }
        if self.validator_address.len() != VALIDATOR_ADDR_SIZE {
            return Err(InvalidValidatorAddressSize.into());
        }

        self.block_id
            .as_ref()
            .map_or(Ok(()), ConsensusMessage::validate_basic)

        // signature will be missing as the KMS provides it
    }
}

#[cfg(test)]
mod tests {
    use super::super::PartSetHeader;
    use super::*;
    use crate::chain::Id;
    use chrono::{DateTime, Utc};
    use tendermint_proto::types::SignedMsgType;
    use tendermint_proto::DomainType;

    #[test]
    fn test_vote_serialization() {
        let dt = "2017-12-25T03:00:01.234Z".parse::<DateTime<Utc>>().unwrap();
        let t = Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        };
        let vote = Vote {
            vote_type: SignedMsgType::Prevote as u16,
            height: 12345,
            round: 2,
            timestamp: Some(t),
            block_id: Some(BlockId {
                hash: b"DEADBEEFDEADBEEFBAFBAFBAFBAFBAFA".to_vec(),
                part_set_header: Some(PartSetHeader {
                    total: 1_000_000,
                    hash: b"0022446688AACCEE1133557799BBDDFF".to_vec(),
                }),
            }),
            validator_address: vec![
                0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
                0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
            ],
            validator_index: 56789,
            signature: vec![],
            /* signature: vec![130u8, 246, 183, 50, 153, 248, 28, 57, 51, 142, 55, 217, 194, 24,
             * 134, 212, 233, 100, 211, 10, 24, 174, 179, 117, 41, 65, 141, 134, 149, 239, 65,
             * 174, 217, 42, 6, 184, 112, 17, 7, 97, 255, 221, 252, 16, 60, 144, 30, 212, 167,
             * 39, 67, 35, 118, 192, 133, 130, 193, 115, 32, 206, 152, 91, 173, 10], */
        };
        let mut got = vec![];

        let request = SignVoteRequest {
            vote: Some(vote),
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
           func voteSerialize() {
               stamp, _ := time.Parse(time.RFC3339Nano, "2017-12-25T03:00:01.234Z")
               vote := &types.Vote{
                   Type:      prototypes.PrevoteType, // pre-vote
                   Height:    12345,
                   Round:     2,
                   Timestamp: stamp,
                   BlockID: types.BlockID{
                       Hash: []byte("DEADBEEFDEADBEEFBAFBAFBAFBAFBAFA"),
                       PartSetHeader: types.PartSetHeader{
                           Total: 1000000,
                           Hash:  []byte("0022446688AACCEE1133557799BBDDFF"),
                       },
                   },
                   ValidatorAddress: []byte{0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21,
                       0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35}, ValidatorIndex: 56789}
               signBytes := types.VoteSignBytes("test_chain_id", vote.ToProto())
               fmt.Println(strings.Join(strings.Split(fmt.Sprintf("%v", signBytes), " "), ", "))
           }
        */

        let want = vec![
            124, 8, 1, 17, 57, 48, 0, 0, 0, 0, 0, 0, 25, 2, 0, 0, 0, 0, 0, 0, 0, 34, 74, 10, 32,
            68, 69, 65, 68, 66, 69, 69, 70, 68, 69, 65, 68, 66, 69, 69, 70, 66, 65, 70, 66, 65, 70,
            66, 65, 70, 66, 65, 70, 66, 65, 70, 65, 18, 38, 8, 192, 132, 61, 18, 32, 48, 48, 50,
            50, 52, 52, 54, 54, 56, 56, 65, 65, 67, 67, 69, 69, 49, 49, 51, 51, 53, 53, 55, 55, 57,
            57, 66, 66, 68, 68, 70, 70, 42, 11, 8, 177, 211, 129, 210, 5, 16, 128, 157, 202, 111,
            50, 13, 116, 101, 115, 116, 95, 99, 104, 97, 105, 110, 95, 105, 100,
        ];
        assert_eq!(got, want);
    }

    #[test]
    fn test_sign_bytes_compatibility() {
        let cv = CanonicalVote::new(Vote::default(), "");
        let mut got = vec![];
        // SignBytes are encoded using MarshalBinary and not MarshalBinaryBare
        cv.encode_length_delimited(&mut got).unwrap();
        let want = vec![
            0xd, 0x2a, 0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1,
        ];
        assert_eq!(got, want);

        // with proper (fixed size) height and round (Precommit):
        {
            let mut vt_precommit = Vote::default();
            vt_precommit.height = 1;
            vt_precommit.round = 1;
            vt_precommit.vote_type = SignedMsgType::Precommit as u16; // precommit
            println!("{:?}", vt_precommit);
            let cv_precommit = CanonicalVote::new(vt_precommit, "");
            //let got = AminoMessage::bytes_vec(&cv_precommit); //Todo: Greg reintroduce Vec<u8>
            // converted encode/decode
            let mut got = vec![];
            cv_precommit.encode(&mut got).unwrap();
            let want = vec![
                0x8,  // (field_number << 3) | wire_type
                0x2,  // PrecommitType
                0x11, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                0x19, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // round
                0x2a, // (field_number << 3) | wire_type
                // remaining fields (timestamp):
                0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1,
            ];
            assert_eq!(got, want);
        }
        // with proper (fixed size) height and round (Prevote):
        {
            let mut vt_prevote = Vote::default();
            vt_prevote.height = 1;
            vt_prevote.round = 1;
            vt_prevote.vote_type = SignedMsgType::Prevote as u16;

            let cv_prevote = CanonicalVote::new(vt_prevote, "");

            //let got = AminoMessage::bytes_vec(&cv_prevote); // Todo: Greg reintroduce Vec<u8>
            // encode.
            let mut got = vec![];
            cv_prevote.encode(&mut got).unwrap();

            let want = vec![
                0x8,  // (field_number << 3) | wire_type
                0x1,  // PrevoteType
                0x11, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                0x19, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // round
                0x2a, // (field_number << 3) | wire_type
                // remaining fields (timestamp):
                0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1,
            ];
            assert_eq!(got, want);
        }
        // with proper (fixed size) height and round (msg typ missing):
        {
            let mut vt_no_type = Vote::default();
            vt_no_type.height = 1;
            vt_no_type.round = 1;

            let cv = CanonicalVote::new(vt_no_type, "");
            //let got = AminoMessage::bytes_vec(&cv);
            let mut got = vec![];
            cv.encode(&mut got).unwrap();

            let want = vec![
                0x11, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                0x19, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // round
                // remaining fields (timestamp):
                0x2a, 0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1,
            ];
            assert_eq!(got, want);
        }
        // containing non-empty chain_id:
        {
            let mut no_vote_type2 = Vote::default();
            no_vote_type2.height = 1;
            no_vote_type2.round = 1;

            let with_chain_id = CanonicalVote::new(no_vote_type2, "test_chain_id");
            //got = AminoMessage::bytes_vec(&with_chain_id);
            let mut got = vec![];
            with_chain_id.encode(&mut got).unwrap();

            let want = vec![
                0x11, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                0x19, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // round
                // remaining fields:
                0x2a, // (field_number << 3) | wire_type
                0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff,
                0x1,  // timestamp
                0x32, // (field_number << 3) | wire_type
                0xd, 0x74, 0x65, 0x73, 0x74, 0x5f, 0x63, 0x68, 0x61, 0x69, 0x6e, 0x5f, 0x69,
                0x64, // chainID
            ];
            assert_eq!(got, want);
        }
    }

    #[test]
    fn test_vote_rountrip_with_sig() {
        let dt = "2017-12-25T03:00:01.234Z".parse::<DateTime<Utc>>().unwrap();
        let t = Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        };
        let vote = Vote {
            validator_address: vec![
                0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
                0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
            ],
            validator_index: 56789,
            height: 12345,
            round: 2,
            timestamp: Some(t),
            vote_type: 0x01,
            block_id: Some(BlockId {
                hash: b"hash".to_vec(),
                part_set_header: Some(PartSetHeader {
                    total: 1_000_000,
                    hash: b"parts_hash".to_vec(),
                }),
            }),
            // signature: None,
            signature: vec![
                130u8, 246, 183, 50, 153, 248, 28, 57, 51, 142, 55, 217, 194, 24, 134, 212, 233,
                100, 211, 10, 24, 174, 179, 117, 41, 65, 141, 134, 149, 239, 65, 174, 217, 42, 6,
                184, 112, 17, 7, 97, 255, 221, 252, 16, 60, 144, 30, 212, 167, 39, 67, 35, 118,
                192, 133, 130, 193, 115, 32, 206, 152, 91, 173, 10,
            ],
        };
        let mut got = vec![];
        let _have = vote.encode(&mut got);
        let v = Vote::decode(got.as_ref()).unwrap();

        assert_eq!(v, vote);
        // SignVoteRequest
        {
            let svr = SignVoteRequest {
                vote: Some(vote),
                chain_id: "test_chain_id".to_string(),
            };
            let mut got = vec![];
            let _have = svr.encode(&mut got);

            let svr2 = SignVoteRequest::decode(got.as_ref()).unwrap();
            assert_eq!(svr, svr2);
        }
    }

    #[test]
    fn test_deserialization() {
        let encoded = vec![
            10, 122, 8, 1, 16, 185, 96, 24, 2, 34, 74, 10, 32, 68, 69, 65, 68, 66, 69, 69, 70, 68,
            69, 65, 68, 66, 69, 69, 70, 66, 65, 70, 66, 65, 70, 66, 65, 70, 66, 65, 70, 66, 65, 70,
            65, 18, 38, 8, 192, 132, 61, 18, 32, 48, 48, 50, 50, 52, 52, 54, 54, 56, 56, 65, 65,
            67, 67, 69, 69, 49, 49, 51, 51, 53, 53, 55, 55, 57, 57, 66, 66, 68, 68, 70, 70, 42, 11,
            8, 177, 211, 129, 210, 5, 16, 128, 157, 202, 111, 50, 20, 163, 178, 204, 221, 113, 134,
            241, 104, 95, 33, 242, 72, 42, 244, 251, 52, 70, 168, 75, 53, 56, 213, 187, 3, 18, 13,
            116, 101, 115, 116, 95, 99, 104, 97, 105, 110, 95, 105, 100,
        ];
        let dt = "2017-12-25T03:00:01.234Z".parse::<DateTime<Utc>>().unwrap();
        let t = Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        };
        let vote = Vote {
            validator_address: vec![
                0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
                0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
            ],
            validator_index: 56789,
            height: 12345,
            round: 2,
            timestamp: Some(t),
            vote_type: 0x01,
            block_id: Some(BlockId {
                hash: b"DEADBEEFDEADBEEFBAFBAFBAFBAFBAFA".to_vec(),
                part_set_header: Some(PartSetHeader {
                    total: 1_000_000,
                    hash: b"0022446688AACCEE1133557799BBDDFF".to_vec(),
                }),
            }),
            signature: vec![],
        };
        let want = SignVoteRequest {
            vote: Some(vote),
            chain_id: "test_chain_id".to_string(),
        };
        match SignVoteRequest::decode(encoded.as_ref()) {
            Ok(have) => {
                assert_eq!(have, want);
            }
            Err(err) => panic!(err.to_string()),
        }
    }
}
