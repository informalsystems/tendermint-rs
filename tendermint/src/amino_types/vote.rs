use super::{
    block_id::{BlockId, CanonicalBlockId, CanonicalPartSetHeader},
    compute_prefix,
    remote_error::RemoteError,
    signature::SignableMsg,
    time::TimeMsg,
    validate::{ConsensusMessage, ValidationError, ValidationErrorKind::*},
    SignedMsgType,
};
use crate::amino_types::PartsSetHeader;
use crate::{
    block::{self, ParseId},
    chain, consensus,
    error::Error,
    vote,
};
use bytes::BufMut;
use once_cell::sync::Lazy;
use prost_amino::{error::EncodeError, Message};
use prost_amino_derive::Message;
use signatory::ed25519;
use std::convert::TryFrom;

const VALIDATOR_ADDR_SIZE: usize = 20;

#[derive(Clone, PartialEq, Message)]
pub struct Vote {
    #[prost_amino(uint32, tag = "1")]
    pub vote_type: u32,
    #[prost_amino(int64)]
    pub height: i64,
    #[prost_amino(int64)]
    pub round: i64,
    #[prost_amino(message)]
    pub block_id: Option<BlockId>,
    #[prost_amino(message)]
    pub timestamp: Option<TimeMsg>,
    #[prost_amino(bytes)]
    pub validator_address: Vec<u8>,
    #[prost_amino(int64)]
    pub validator_index: i64,
    #[prost_amino(bytes)]
    pub signature: Vec<u8>,
}

impl Vote {
    fn msg_type(&self) -> Option<SignedMsgType> {
        if self.vote_type == SignedMsgType::PreVote.to_u32() {
            Some(SignedMsgType::PreVote)
        } else if self.vote_type == SignedMsgType::PreCommit.to_u32() {
            Some(SignedMsgType::PreCommit)
        } else {
            None
        }
    }
}

impl From<&vote::Vote> for Vote {
    fn from(vote: &vote::Vote) -> Self {
        Vote {
            vote_type: vote.vote_type.to_u32(),
            height: vote.height.value() as i64, // TODO potential overflow :-/
            round: vote.round as i64,
            block_id: vote.block_id.as_ref().map(|block_id| BlockId {
                hash: block_id.hash.as_bytes().to_vec(),
                parts_header: block_id.parts.as_ref().map(PartsSetHeader::from),
            }),
            timestamp: Some(TimeMsg::from(vote.timestamp)),
            validator_address: vote.validator_address.as_bytes().to_vec(),
            validator_index: vote.validator_index as i64, // TODO potential overflow :-/
            signature: vote.signature.as_bytes().to_vec(),
        }
    }
}

impl block::ParseHeight for Vote {
    fn parse_block_height(&self) -> Result<block::Height, Error> {
        block::Height::try_from(self.height)
    }
}

pub const AMINO_NAME: &str = "tendermint/remotesigner/SignVoteRequest";
pub static AMINO_PREFIX: Lazy<Vec<u8>> = Lazy::new(|| compute_prefix(AMINO_NAME));

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/remotesigner/SignVoteRequest"]
pub struct SignVoteRequest {
    #[prost_amino(message, tag = "1")]
    pub vote: Option<Vote>,
}

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/remotesigner/SignedVoteResponse"]
pub struct SignedVoteResponse {
    #[prost_amino(message, tag = "1")]
    pub vote: Option<Vote>,
    #[prost_amino(message, tag = "2")]
    pub err: Option<RemoteError>,
}

#[derive(Clone, PartialEq, Message)]
pub struct CanonicalVote {
    #[prost_amino(uint32, tag = "1")]
    pub vote_type: u32,
    #[prost_amino(sfixed64)]
    pub height: i64,
    #[prost_amino(sfixed64)]
    pub round: i64,
    #[prost_amino(message)]
    pub block_id: Option<CanonicalBlockId>,
    #[prost_amino(message)]
    pub timestamp: Option<TimeMsg>,
    #[prost_amino(string)]
    pub chain_id: String,
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
                    parts_header: match bid.parts_header {
                        Some(psh) => Some(CanonicalPartSetHeader {
                            hash: psh.hash,
                            total: psh.total,
                        }),
                        None => None,
                    },
                }),
                None => None,
            },
            height: vote.height,
            round: vote.round,
            timestamp: match vote.timestamp {
                None => Some(TimeMsg {
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

        cv.encode_length_delimited(sign_bytes)?;

        Ok(true)
    }
    fn set_signature(&mut self, sig: &ed25519::Signature) {
        if let Some(ref mut vt) = self.vote {
            vt.signature = sig.as_ref().to_vec();
        }
    }
    fn validate(&self) -> Result<(), ValidationError> {
        match self.vote {
            Some(ref v) => v.validate_basic(),
            None => Err(MissingConsensusMessage.into()),
        }
    }
    fn consensus_state(&self) -> Option<consensus::State> {
        match self.vote {
            Some(ref v) => Some(consensus::State {
                height: match block::Height::try_from(v.height) {
                    Ok(h) => h,
                    Err(_err) => return None, // TODO(tarcieri): return an error?
                },
                round: v.round,
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
        self.vote.as_ref().map(|vote| vote.height)
    }
    fn msg_type(&self) -> Option<SignedMsgType> {
        self.vote.as_ref().and_then(|vote| vote.msg_type())
    }
}

impl ConsensusMessage for Vote {
    fn validate_basic(&self) -> Result<(), ValidationError> {
        if self.msg_type().is_none() {
            return Err(InvalidMessageType.into());
        }
        if self.height < 0 {
            return Err(NegativeHeight.into());
        }
        if self.round < 0 {
            return Err(NegativeRound.into());
        }
        if self.validator_index < 0 {
            return Err(NegativeValidatorIndex.into());
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
    use super::super::PartsSetHeader;
    use super::*;
    use crate::amino_types::message::AminoMessage;
    use crate::amino_types::SignedMsgType;
    use chrono::{DateTime, Utc};

    #[test]
    fn test_vote_serialization() {
        let dt = "2017-12-25T03:00:01.234Z".parse::<DateTime<Utc>>().unwrap();
        let t = TimeMsg {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        };
        let vote = Vote {
            vote_type: SignedMsgType::PreVote.to_u32(),
            height: 12345,
            round: 2,
            timestamp: Some(t),
            block_id: Some(BlockId {
                hash: b"hash".to_vec(),
                parts_header: Some(PartsSetHeader {
                    total: 1_000_000,
                    hash: b"parts_hash".to_vec(),
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
        let sign_vote_msg = SignVoteRequest { vote: Some(vote) };
        let mut got = vec![];
        let _have = sign_vote_msg.encode(&mut got);

        // the following vector is generated via:
        //
        // cdc := amino.NewCodec()
        // privval.RegisterRemoteSignerMsg(cdc)
        // stamp, _ := time.Parse(time.RFC3339Nano, "2017-12-25T03:00:01.234Z")
        // data, _ := cdc.MarshalBinaryLengthPrefixed(privval.SignVoteRequest{Vote: &types.Vote{
        //     Type:             types.PrevoteType, // pre-vote
        //     Height:           12345,
        //     Round:            2,
        //     Timestamp:        stamp,
        //     BlockID: types.BlockID{
        //         Hash: []byte("hash"),
        //         PartsHeader: types.PartSetHeader{
        //             Total: 1000000,
        //             Hash:  []byte("parts_hash"),
        //         },
        //     },
        //     ValidatorAddress: []byte{0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21,
        // 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35},     ValidatorIndex:
        // 56789, }})
        // fmt.Println(strings.Join(strings.Split(fmt.Sprintf("%v",data), " "), ", "))

        let want = vec![
            78, 243, 244, 18, 4, 10, 72, 8, 1, 16, 185, 96, 24, 2, 34, 24, 10, 4, 104, 97, 115,
            104, 18, 16, 8, 192, 132, 61, 18, 10, 112, 97, 114, 116, 115, 95, 104, 97, 115, 104,
            42, 11, 8, 177, 211, 129, 210, 5, 16, 128, 157, 202, 111, 50, 20, 163, 178, 204, 221,
            113, 134, 241, 104, 95, 33, 242, 72, 42, 244, 251, 52, 70, 168, 75, 53, 56, 213, 187,
            3,
        ];
        let svr = SignVoteRequest::decode(got.as_ref()).unwrap();
        println!("got back: {:?}", svr);
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

        // with proper (fixed size) height and round (PreCommit):
        {
            let mut vt_precommit = Vote::default();
            vt_precommit.height = 1;
            vt_precommit.round = 1;
            vt_precommit.vote_type = SignedMsgType::PreCommit.to_u32(); // precommit
            println!("{:?}", vt_precommit);
            let cv_precommit = CanonicalVote::new(vt_precommit, "");
            let got = AminoMessage::bytes_vec(&cv_precommit);
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
        // with proper (fixed size) height and round (PreVote):
        {
            let mut vt_prevote = Vote::default();
            vt_prevote.height = 1;
            vt_prevote.round = 1;
            vt_prevote.vote_type = SignedMsgType::PreVote.to_u32();

            let cv_prevote = CanonicalVote::new(vt_prevote, "");

            let got = AminoMessage::bytes_vec(&cv_prevote);

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
            let got = AminoMessage::bytes_vec(&cv);

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
            got = AminoMessage::bytes_vec(&with_chain_id);
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
        let t = TimeMsg {
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
                parts_header: Some(PartsSetHeader {
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
            let svr = SignVoteRequest { vote: Some(vote) };
            let mut got = vec![];
            let _have = svr.encode(&mut got);

            let svr2 = SignVoteRequest::decode(got.as_ref()).unwrap();
            assert_eq!(svr, svr2);
        }
    }

    #[test]
    fn test_deserialization() {
        let encoded = vec![
            78, 243, 244, 18, 4, 10, 72, 8, 1, 16, 185, 96, 24, 2, 34, 24, 10, 4, 104, 97, 115,
            104, 18, 16, 8, 192, 132, 61, 18, 10, 112, 97, 114, 116, 115, 95, 104, 97, 115, 104,
            42, 11, 8, 177, 211, 129, 210, 5, 16, 128, 157, 202, 111, 50, 20, 163, 178, 204, 221,
            113, 134, 241, 104, 95, 33, 242, 72, 42, 244, 251, 52, 70, 168, 75, 53, 56, 213, 187,
            3,
        ];
        let dt = "2017-12-25T03:00:01.234Z".parse::<DateTime<Utc>>().unwrap();
        let t = TimeMsg {
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
                parts_header: Some(PartsSetHeader {
                    total: 1_000_000,
                    hash: b"parts_hash".to_vec(),
                }),
            }),
            signature: vec![],
        };
        let want = SignVoteRequest { vote: Some(vote) };
        match SignVoteRequest::decode(encoded.as_ref()) {
            Ok(have) => {
                assert_eq!(have, want);
            }
            Err(err) => panic!(err.to_string()),
        }
    }
}
