use crate::chain;
use crate::error::Error;
use crate::prelude::*;
use crate::Vote;
use bytes::BufMut;
use core::convert::{TryFrom, TryInto};
use tendermint_proto::privval::SignedVoteResponse as RawSignedVoteResponse;
use tendermint_proto::privval::{RemoteSignerError, SignVoteRequest as RawSignVoteRequest};
use tendermint_proto::Error as ProtobufError;
use tendermint_proto::Protobuf;

/// SignVoteRequest is a request to sign a vote
#[derive(Clone, PartialEq, Debug)]
pub struct SignVoteRequest {
    /// Vote
    pub vote: Vote,
    /// Chain ID
    pub chain_id: chain::Id,
}

impl Protobuf<RawSignVoteRequest> for SignVoteRequest {}

impl TryFrom<RawSignVoteRequest> for SignVoteRequest {
    type Error = Error;

    fn try_from(value: RawSignVoteRequest) -> Result<Self, Self::Error> {
        let vote = value.vote.ok_or_else(Error::no_vote_found)?.try_into()?;

        let chain_id = value.chain_id.try_into()?;

        Ok(SignVoteRequest { vote, chain_id })
    }
}

impl From<SignVoteRequest> for RawSignVoteRequest {
    fn from(value: SignVoteRequest) -> Self {
        RawSignVoteRequest {
            vote: Some(value.vote.into()),
            chain_id: value.chain_id.as_str().to_string(),
        }
    }
}

impl SignVoteRequest {
    /// Create signable bytes from Vote.
    pub fn to_signable_bytes<B>(&self, sign_bytes: &mut B) -> Result<bool, ProtobufError>
    where
        B: BufMut,
    {
        self.vote
            .to_signable_bytes(self.chain_id.clone(), sign_bytes)
    }

    /// Create signable vector from Vote.
    pub fn to_signable_vec(&self) -> Result<Vec<u8>, ProtobufError> {
        self.vote.to_signable_vec(self.chain_id.clone())
    }
}

/// SignedVoteResponse is a response containing a signed vote or an error
#[derive(Clone, PartialEq, Debug)]
pub struct SignedVoteResponse {
    /// Optional Vote
    pub vote: Option<Vote>,
    /// Optional error
    pub error: Option<RemoteSignerError>,
}

impl Protobuf<RawSignedVoteResponse> for SignedVoteResponse {}

impl TryFrom<RawSignedVoteResponse> for SignedVoteResponse {
    type Error = Error;

    fn try_from(value: RawSignedVoteResponse) -> Result<Self, Self::Error> {
        Ok(SignedVoteResponse {
            vote: value.vote.map(TryFrom::try_from).transpose()?,
            error: value.error,
        })
    }
}

impl From<SignedVoteResponse> for RawSignedVoteResponse {
    fn from(value: SignedVoteResponse) -> Self {
        RawSignedVoteResponse {
            vote: value.vote.map(Into::into),
            error: value.error,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::account::Id as AccountId;
    use crate::block::parts::Header;
    use crate::block::Height;
    use crate::block::Id as BlockId;
    use crate::block::Round;
    use crate::chain::Id as ChainId;
    use crate::hash::Algorithm;
    use crate::prelude::*;
    use crate::signature::{Ed25519Signature, Signature};
    use crate::vote::{CanonicalVote, ValidatorIndex};
    use crate::vote::{SignVoteRequest, Type};
    use crate::Hash;
    use crate::Vote;
    use core::convert::{TryFrom, TryInto};
    use core::str::FromStr;
    use std::println;
    use tendermint_proto::Protobuf;
    use time::macros::datetime;

    #[test]
    fn test_vote_serialization() {
        let dt = datetime!(2017-12-25 03:00:01.234 UTC);
        let vote = Vote {
            vote_type: Type::Prevote,
            height: Height::from(12345_u32),
            round: Round::from(2_u16),
            timestamp: Some(dt.try_into().unwrap()),
            block_id: Some(BlockId {
                hash: Hash::try_from(b"DEADBEEFDEADBEEFBAFBAFBAFBAFBAFA".to_vec()).unwrap(),
                part_set_header: Header::new(
                    1_000_000,
                    Hash::try_from(b"0022446688AACCEE1133557799BBDDFF".to_vec()).unwrap(),
                )
                .unwrap(),
            }),
            validator_address: AccountId::try_from(vec![
                0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
                0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
            ])
            .unwrap(),
            validator_index: ValidatorIndex::try_from(56789).unwrap(),
            signature: Signature::new(vec![
                130u8, 246, 183, 50, 153, 248, 28, 57, 51, 142, 55, 217, 194, 24, 134, 212, 233,
                100, 211, 10, 24, 174, 179, 117, 41, 65, 141, 134, 149, 239, 65, 174, 217, 42, 6,
                184, 112, 17, 7, 97, 255, 221, 252, 16, 60, 144, 30, 212, 167, 39, 67, 35, 118,
                192, 133, 130, 193, 115, 32, 206, 152, 91, 173, 10,
            ])
            .unwrap(),
        };

        let mut got = vec![];

        let request = SignVoteRequest {
            vote,
            chain_id: ChainId::from_str("test_chain_id").unwrap(),
        };

        // Option 1 using bytes:
        let _have = request.to_signable_bytes(&mut got);
        // Option 2 using Vec<u8>:
        let got2 = request.to_signable_vec().unwrap();

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
        assert_eq!(got2, want);
    }

    #[test]
    // Test vote encoding with a malformed block_id (no hash) which is considered nil in Go.
    fn test_vote_encoding_with_empty_block_id() {
        let dt = datetime!(2017-12-25 03:00:01.234 UTC);
        let vote = Vote {
            vote_type: Type::Prevote,
            height: Height::from(12345_u32),
            round: Round::from(2_u16),
            timestamp: Some(dt.try_into().unwrap()),
            block_id: Some(BlockId {
                hash: Hash::try_from(b"".to_vec()).unwrap(),
                part_set_header: Header::new(
                    1_000_000,
                    Hash::try_from(b"0022446688AACCEE1133557799BBDDFF".to_vec()).unwrap(),
                )
                .unwrap(),
            }),
            validator_address: AccountId::try_from(vec![
                0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
                0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
            ])
            .unwrap(),
            validator_index: ValidatorIndex::try_from(56789).unwrap(),
            signature: Signature::new(vec![
                130u8, 246, 183, 50, 153, 248, 28, 57, 51, 142, 55, 217, 194, 24, 134, 212, 233,
                100, 211, 10, 24, 174, 179, 117, 41, 65, 141, 134, 149, 239, 65, 174, 217, 42, 6,
                184, 112, 17, 7, 97, 255, 221, 252, 16, 60, 144, 30, 212, 167, 39, 67, 35, 118,
                192, 133, 130, 193, 115, 32, 206, 152, 91, 173, 10,
            ])
            .unwrap(),
        };

        let request = SignVoteRequest {
            vote,
            chain_id: ChainId::from_str("test_chain_id").unwrap(),
        };

        let got = request.to_signable_vec().unwrap();

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
                       Hash: []byte(""),
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
            90, 8, 1, 17, 57, 48, 0, 0, 0, 0, 0, 0, 25, 2, 0, 0, 0, 0, 0, 0, 0, 34, 40, 18, 38, 8,
            192, 132, 61, 18, 32, 48, 48, 50, 50, 52, 52, 54, 54, 56, 56, 65, 65, 67, 67, 69, 69,
            49, 49, 51, 51, 53, 53, 55, 55, 57, 57, 66, 66, 68, 68, 70, 70, 42, 11, 8, 177, 211,
            129, 210, 5, 16, 128, 157, 202, 111, 50, 13, 116, 101, 115, 116, 95, 99, 104, 97, 105,
            110, 95, 105, 100,
        ];
        assert_eq!(got, want);
    }

    #[test]
    fn test_sign_bytes_compatibility() {
        let cv = CanonicalVote::new(Vote::default(), ChainId::try_from("A").unwrap());
        let mut got = vec![];
        // SignBytes are encoded using MarshalBinary and not MarshalBinaryBare
        cv.encode_length_delimited(&mut got).unwrap();
        let want = vec![
            0x10, 0x8, 0x1, 0x11, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2a, 0x0, 0x32, 0x1,
            0x41,
        ]; // Todo: Get these bytes from Go. During protobuf upgrade we didn't get to generate them.
        assert_eq!(got, want);

        // with proper (fixed size) height and round (Precommit):
        {
            let vt_precommit = Vote {
                height: Height::from(1_u32),
                round: Round::from(1_u16),
                vote_type: Type::Precommit,
                ..Default::default()
            };
            println!("{:?}", vt_precommit);
            let cv_precommit = CanonicalVote::new(vt_precommit, ChainId::try_from("A").unwrap());
            let got = cv_precommit.encode_vec().unwrap();
            let want = vec![
                0x8,  // (field_number << 3) | wire_type
                0x2,  // PrecommitType
                0x11, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                0x19, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // round
                0x2a, // (field_number << 3) | wire_type
                0x0,  // timestamp
                0x32, // (field_number << 3) | wire_type
                // remaining fields (chain ID):
                0x1, 0x41,
            ];
            assert_eq!(got, want);
        }
        // with proper (fixed size) height and round (Prevote):
        {
            let vt_prevote = Vote {
                height: Height::from(1_u32),
                round: Round::from(1_u16),
                vote_type: Type::Prevote,
                ..Default::default()
            };

            let cv_prevote = CanonicalVote::new(vt_prevote, ChainId::try_from("A").unwrap());

            let got = cv_prevote.encode_vec().unwrap();

            let want = vec![
                0x8,  // (field_number << 3) | wire_type
                0x1,  // PrevoteType
                0x11, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                0x19, // (field_number << 3) | wire_type
                0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // round
                0x2a, // (field_number << 3) | wire_type
                0x0,  // timestamp
                0x32, // (field_number << 3) | wire_type
                // remaining fields (chain ID):
                0x1, 0x41,
            ];
            assert_eq!(got, want);
        }
    }

    #[test]
    fn test_vote_rountrip_with_sig() {
        let dt = datetime!(2017-12-25 03:00:01.234 UTC);
        let vote = Vote {
            validator_address: AccountId::try_from(vec![
                0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
                0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
            ])
            .unwrap(),
            validator_index: ValidatorIndex::try_from(56789).unwrap(),
            height: Height::from(12345_u32),
            round: Round::from(2_u16),
            timestamp: Some(dt.try_into().unwrap()),
            vote_type: Type::Prevote,
            block_id: Some(BlockId {
                hash: Hash::from_hex_upper(Algorithm::Sha256, "DEADBEEFDEADBEEFBAFBAFBAFBAFBAFA")
                    .unwrap(), //Hash::new(Algorithm::Sha256,
                // b"hash".to_vec().as_slice()).unwrap(),
                part_set_header: Header::new(
                    1_000_000,
                    Hash::from_hex_upper(Algorithm::Sha256, "DEADBEEFDEADBEEFBAFBAFBAFBAFBAFA")
                        .unwrap(),
                )
                .unwrap(),
            }),
            // signature: None,
            signature: Signature::new(vec![
                130u8, 246, 183, 50, 153, 248, 28, 57, 51, 142, 55, 217, 194, 24, 134, 212, 233,
                100, 211, 10, 24, 174, 179, 117, 41, 65, 141, 134, 149, 239, 65, 174, 217, 42, 6,
                184, 112, 17, 7, 97, 255, 221, 252, 16, 60, 144, 30, 212, 167, 39, 67, 35, 118,
                192, 133, 130, 193, 115, 32, 206, 152, 91, 173, 10,
            ])
            .unwrap(),
        };
        let got = vote.encode_vec().unwrap();
        let v = Vote::decode_vec(&got).unwrap();

        assert_eq!(v, vote);
        // SignVoteRequest
        {
            let svr = SignVoteRequest {
                vote,
                chain_id: ChainId::from_str("test_chain_id").unwrap(),
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
            10, 188, 1, 8, 1, 16, 185, 96, 24, 2, 34, 74, 10, 32, 222, 173, 190, 239, 222, 173,
            190, 239, 186, 251, 175, 186, 251, 175, 186, 250, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 18, 38, 8, 192, 132, 61, 18, 32, 0, 34, 68, 102, 136, 170, 204, 238, 17,
            51, 85, 119, 153, 187, 221, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 42,
            11, 8, 177, 211, 129, 210, 5, 16, 128, 157, 202, 111, 50, 20, 163, 178, 204, 221, 113,
            134, 241, 104, 95, 33, 242, 72, 42, 244, 251, 52, 70, 168, 75, 53, 56, 213, 187, 3, 66,
            64, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 18, 13, 116, 101, 115, 116, 95, 99, 104, 97, 105, 110, 95, 105,
            100,
        ]; // Todo: Double-check the Go implementation, this was self-generated.
        let dt = datetime!(2017-12-25 03:00:01.234 UTC);
        let vote = Vote {
            validator_address: AccountId::try_from(vec![
                0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
                0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
            ])
            .unwrap(),
            validator_index: ValidatorIndex::try_from(56789).unwrap(),
            height: Height::from(12345_u32),
            round: Round::from(2_u16),
            timestamp: Some(dt.try_into().unwrap()),
            vote_type: Type::Prevote,
            block_id: Some(BlockId {
                hash: Hash::from_hex_upper(Algorithm::Sha256, "DEADBEEFDEADBEEFBAFBAFBAFBAFBAFA")
                    .unwrap(),
                part_set_header: Header::new(
                    1_000_000,
                    Hash::from_hex_upper(Algorithm::Sha256, "0022446688AACCEE1133557799BBDDFF")
                        .unwrap(),
                )
                .unwrap(),
            }),
            signature: Signature::new(vec![1; Ed25519Signature::BYTE_SIZE]).unwrap(),
        };
        let want = SignVoteRequest {
            vote,
            chain_id: ChainId::from_str("test_chain_id").unwrap(),
        };
        let got = SignVoteRequest::decode_vec(&encoded).unwrap();
        assert_eq!(got, want);
    }
}
