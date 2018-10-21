use super::{
    BlockID, CanonicalBlockID, CanonicalPartSetHeader, Ed25519Signature, RemoteError, Signature,
    TendermintSignable, Time,
};
use bytes::BufMut;
use prost::Message;
use types::prost_amino::error::EncodeError;

#[derive(Clone, PartialEq, Message)]
pub struct Vote {
    #[prost(bytes, tag = "1")]
    pub validator_address: Vec<u8>,
    #[prost(sint64)]
    pub validator_index: i64,
    #[prost(sint64)]
    pub height: i64,
    #[prost(sint64)]
    pub round: i64,
    #[prost(message)]
    pub timestamp: Option<Time>,
    #[prost(uint32)]
    pub vote_type: u32,
    #[prost(message)]
    pub block_id: Option<BlockID>,
    #[prost(bytes)]
    pub signature: Vec<u8>,
}

pub const AMINO_NAME: &str = "tendermint/remotesigner/SignVoteRequest";

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/remotesigner/SignVoteRequest"]
pub struct SignVoteRequest {
    #[prost(message, tag = "1")]
    pub vote: Option<Vote>,
}

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/remotesigner/SignedVoteResponse"]
pub struct SignedVoteResponse {
    #[prost(message, tag = "1")]
    pub vote: Option<Vote>,
    #[prost(message, tag = "2")]
    pub err: Option<RemoteError>,
}

#[derive(Clone, PartialEq, Message)]
pub struct CanonicalVote {
    #[prost(uint32, tag = "1")]
    pub vote_type: u32,
    #[prost(sfixed64)]
    pub height: i64,
    #[prost(sfixed64)]
    pub round: i64,
    #[prost(message)]
    pub timestamp: Option<Time>,
    #[prost(message)]
    pub block_id: Option<CanonicalBlockID>,
    #[prost(string)]
    pub chain_id: String,
}

impl CanonicalVote {
    fn new(vote: Vote, chain_id: &str) -> CanonicalVote {
        CanonicalVote {
            vote_type: vote.vote_type,
            chain_id: chain_id.to_string(),
            block_id: match vote.block_id {
                Some(bid) => Some(CanonicalBlockID {
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
                None => Some(Time {
                    seconds: -62_135_596_800,
                    nanos: 0,
                }),
                Some(t) => Some(t),
            },
        }
    }
}

impl TendermintSignable for SignVoteRequest {
    fn sign_bytes<B>(&self, chain_id: &str, sign_bytes: &mut B) -> Result<bool, EncodeError>
    where
        B: BufMut,
    {
        let mut svr = self.clone();
        if let Some(ref mut vo) = svr.vote {
            vo.signature = vec![];
        }
        let vote = svr.vote.unwrap();
        let cv = CanonicalVote::new(vote, chain_id);

        cv.encode(sign_bytes)?;
        Ok(true)
    }
    fn set_signature(&mut self, sig: &Ed25519Signature) {
        if let Some(ref mut vt) = self.vote {
            vt.signature = sig.clone().into_vec();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::PartsSetHeader;
    use super::*;
    use chrono::{DateTime, Utc};
    use types::prost_amino::Message;
    use types::SignedMsgType;

    #[test]
    fn test_vote_serialization() {
        let dt = "2017-12-25T03:00:01.234Z".parse::<DateTime<Utc>>().unwrap();
        let t = Time {
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
            vote_type: SignedMsgType::PreVote.to_u32(),
            block_id: Some(BlockID {
                hash: "hash".as_bytes().to_vec(),
                parts_header: Some(PartsSetHeader {
                    total: 1000000,
                    hash: "parts_hash".as_bytes().to_vec(),
                }),
            }),
            signature: vec![],
            //signature: vec![130u8, 246, 183, 50, 153, 248, 28, 57, 51, 142, 55, 217, 194, 24, 134, 212, 233, 100, 211, 10, 24, 174, 179, 117, 41, 65, 141, 134, 149, 239, 65, 174, 217, 42, 6, 184, 112, 17, 7, 97, 255, 221, 252, 16, 60, 144, 30, 212, 167, 39, 67, 35, 118, 192, 133, 130, 193, 115, 32, 206, 152, 91, 173, 10],
        };
        let sign_vote_msg = SignVoteRequest { vote: Some(vote) };
        let mut got = vec![];
        let _have = sign_vote_msg.encode(&mut got);

        // the following vector is generated via:
        //  cdc := amino.NewCodec()
        //	cdc.RegisterInterface((*privval.SocketPVMsg)(nil), nil)
        //	cdc.RegisterInterface((*crypto.Signature)(nil), nil)
        //	cdc.RegisterConcrete(crypto.SignatureEd25519{},
        //		"tendermint/SignatureEd25519", nil)
        //
        //	cdc.RegisterConcrete(&privval.PubKeyMsg{}, "tendermint/remotesigner/PubKeyMsg", nil)
        //	cdc.RegisterConcrete(&privval.SignVoteMsg{}, "tendermint/remotesigner/SignVoteMsg", nil)
        //	cdc.RegisterConcrete(&privval.SignProposalMsg{}, "tendermint/remotesigner/SignProposalMsg", nil)
        //	cdc.RegisterConcrete(&privval.SignHeartbeatMsg{}, "tendermint/remotesigner/SignHeartbeatMsg", nil)
        //	data, _ := cdc.MarshalBinary(privval.SignVoteMsg{Vote: vote})
        //
        // where vote is equal to
        //
        //  types.Vote{
        //		ValidatorAddress: []byte{0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35},
        //		ValidatorIndex:   56789,
        //		Height:           12345,
        //		Round:            2,
        //		Timestamp:        stamp,
        //		Type:             byte(0x01), // pre-vote
        //		BlockID: types.BlockID{
        //			Hash: []byte("hash"),
        //			PartsHeader: types.PartSetHeader{
        //				Total: 1000000,
        //				Hash:  []byte("parts_hash"),
        //			},
        //		},
        //	}
        let want = vec![
            0x52, 243, 244, 18, 4, 0xa, 0x4c, 0xa, 0x14, 0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1,
            0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35, 0x10,
            0xaa, 0xf7, 0x6, 0x18, 0xf2, 0xc0, 0x1, 0x20, 0x4, 0x2a, 0xe, 0x9, 0xb1, 0x69, 0x40,
            0x5a, 0x0, 0x0, 0x0, 0x0, 0x15, 0x80, 0x8e, 0xf2, 0xd, 0x30, 0x1, 0x3a, 0x18, 0xa, 0x4,
            0x68, 0x61, 0x73, 0x68, 0x12, 0x10, 0x8, 0x80, 0x89, 0x7a, 0x12, 0xa, 0x70, 0x61, 0x72,
            0x74, 0x73, 0x5f, 0x68, 0x61, 0x73, 0x68,
        ];
        let svr = SignVoteRequest::decode(got.clone()).unwrap();
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
            0xb, 0x22, 0x9, 0x9, 0x0, 0x9, 0x6e, 0x88, 0xf1, 0xff, 0xff, 0xff,
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
            got = vec![];
            cv_precommit.encode_length_delimited(&mut got).unwrap();
            let want = vec![
                0x1f, // total length
                0x8,  // (field_number << 3) | wire_type
                0x2,  // PrecommitType
                0x11, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                0x19, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // round
                0x22, // (field_number << 3) | wire_type
                // remaining fields (timestamp):
                0x9, 0x9, 0x0, 0x9, 0x6e, 0x88, 0xf1, 0xff, 0xff, 0xff,
            ];
            assert_eq!(got, want);
        }
        // with proper (fixed size) height and round (PreVote):
        {
            let mut vt_prevote = Vote::default();
            vt_prevote.height = 1;
            vt_prevote.round = 1;
            vt_prevote.vote_type = SignedMsgType::PreVote.to_u32();

            got = vec![];
            let cv_prevote = CanonicalVote::new(vt_prevote, "");

            cv_prevote.encode_length_delimited(&mut got).unwrap();

            let want = vec![
                0x1f, // total length
                0x8,  // (field_number << 3) | wire_type
                0x1,  // PrevoteType
                0x11, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                0x19, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // round
                0x22, // (field_number << 3) | wire_type
                // remaining fields (timestamp):
                0x9, 0x9, 0x0, 0x9, 0x6e, 0x88, 0xf1, 0xff, 0xff, 0xff,
            ];
            assert_eq!(got, want);
        }
        // with proper (fixed size) height and round (msg typ missing):
        {
            let mut vt_no_type = Vote::default();
            vt_no_type.height = 1;
            vt_no_type.round = 1;

            got = vec![];
            let cv = CanonicalVote::new(vt_no_type, "");
            cv.encode_length_delimited(&mut got).unwrap();

            let want = vec![
                0x1d, // total length
                0x11, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                0x19, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // round
                // remaining fields (timestamp):
                0x22, 0x9, 0x9, 0x0, 0x9, 0x6e, 0x88, 0xf1, 0xff, 0xff, 0xff,
            ];
            assert_eq!(got, want);
        }
        // containing non-empty chain_id:
        {
            let mut no_vote_type2 = Vote::default();
            no_vote_type2.height = 1;
            no_vote_type2.round = 1;

            let with_chain_id = CanonicalVote::new(no_vote_type2, "test_chain_id");
            got = vec![];
            with_chain_id.encode_length_delimited(&mut got).unwrap();
            let want = vec![
                0x2c, // total length
                0x11, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                0x19, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // round
                // remaining fields:
                0x22, // (field_number << 3) | wire_type
                0x9, 0x9, 0x0, 0x9, 0x6e, 0x88, 0xf1, 0xff, 0xff, 0xff, // timestamp
                0x32, // (field_number << 3) | wire_type
                0xd, 0x74, 0x65, 0x73, 0x74, 0x5f, 0x63, 0x68, 0x61, 0x69, 0x6e, 0x5f, 0x69, 0x64,
            ];
            assert_eq!(got, want);
        }
    }

    #[test]
    fn test_vote_rountrip_with_sig() {
        let dt = "2017-12-25T03:00:01.234Z".parse::<DateTime<Utc>>().unwrap();
        let t = Time {
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
            block_id: Some(BlockID {
                hash: "hash".as_bytes().to_vec(),
                parts_header: Some(PartsSetHeader {
                    total: 1000000,
                    hash: "parts_hash".as_bytes().to_vec(),
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
        let v = Vote::decode(&got).unwrap();

        assert_eq!(v, vote);
        // SignVoteRequest
        {
            let svr = SignVoteRequest { vote: Some(vote) };
            let mut got = vec![];
            let _have = svr.encode(&mut got);

            let svr2 = SignVoteRequest::decode(&got).unwrap();
            assert_eq!(svr, svr2);
        }
    }

    #[test]
    fn test_deserialization() {
        let encoded = vec![
            0x52, 0x2f, 0x62, 0x2d, 0xa6, 0xa, 0x4c, 0xa, 0x14, 0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86,
            0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
            0x10, 0xaa, 0xf7, 0x6, 0x18, 0xf2, 0xc0, 0x1, 0x20, 0x4, 0x2a, 0xe, 0x9, 0xb1, 0x69,
            0x40, 0x5a, 0x0, 0x0, 0x0, 0x0, 0x15, 0x80, 0x8e, 0xf2, 0xd, 0x30, 0x1, 0x3a, 0x18,
            0xa, 0x4, 0x68, 0x61, 0x73, 0x68, 0x12, 0x10, 0x8, 0x80, 0x89, 0x7a, 0x12, 0xa, 0x70,
            0x61, 0x72, 0x74, 0x73, 0x5f, 0x68, 0x61, 0x73, 0x68,
        ];
        let dt = "2017-12-25T03:00:01.234Z".parse::<DateTime<Utc>>().unwrap();
        let t = Time {
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
            block_id: Some(BlockID {
                hash: "hash".as_bytes().to_vec(),
                parts_header: Some(PartsSetHeader {
                    total: 1000000,
                    hash: "parts_hash".as_bytes().to_vec(),
                }),
            }),
            signature: vec![],
        };
        let want = SignVoteRequest { vote: Some(vote) };
        match SignVoteRequest::decode(&encoded) {
            Ok(have) => {
                assert_eq!(have, want);
            }
            Err(err) => assert!(false, err.to_string()),
        }
    }
}
