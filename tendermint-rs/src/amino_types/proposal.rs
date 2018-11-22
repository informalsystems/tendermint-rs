use bytes::BufMut;
use prost::{EncodeError, Message};
use signatory::{ed25519, Signature};

use super::{
    block_id::{BlockId, CanonicalBlockId, CanonicalPartSetHeader},
    remote_error::RemoteError,
    signature::{SignableMsg, SignedMsgType},
    time::TimeMsg,
    validate::{ConsensusMessage, ValidationError, ValidationErrorKind},
};
use block;
use chain;
use error::Error;

#[derive(Clone, PartialEq, Message)]
pub struct Proposal {
    #[prost(uint32, tag = "1")]
    pub msg_type: u32,
    #[prost(int64)]
    pub height: i64,
    #[prost(int64)]
    pub round: i64,
    #[prost(int64)]
    pub pol_round: i64,
    #[prost(message)]
    pub block_id: Option<BlockId>,
    #[prost(message)]
    pub timestamp: Option<TimeMsg>,
    #[prost(bytes)]
    pub signature: Vec<u8>,
}

// TODO(tony): custom derive proc macro for this e.g. `derive(ParseBlockHeight)`
impl block::ParseHeight for Proposal {
    fn parse_block_height(&self) -> Result<block::Height, Error> {
        block::Height::parse(self.height)
    }
}

pub const AMINO_NAME: &str = "tendermint/remotesigner/SignProposalRequest";

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/remotesigner/SignProposalRequest"]
pub struct SignProposalRequest {
    #[prost(message, tag = "1")]
    pub proposal: Option<Proposal>,
}

#[derive(Clone, PartialEq, Message)]
struct CanonicalProposal {
    #[prost(uint32, tag = "1")]
    msg_type: u32, // this is a byte in golang, which is a varint encoded UInt8 (using amino's EncodeUvarint)
    #[prost(sfixed64)]
    height: i64,
    #[prost(sfixed64)]
    round: i64,
    #[prost(sfixed64)]
    pol_round: i64,
    #[prost(message)]
    block_id: Option<CanonicalBlockId>,
    #[prost(message)]
    timestamp: Option<TimeMsg>,
    #[prost(string)]
    pub chain_id: String,
}

impl chain::ParseId for CanonicalProposal {
    fn parse_chain_id(&self) -> Result<chain::Id, Error> {
        chain::Id::new(&self.chain_id)
    }
}

impl block::ParseHeight for CanonicalProposal {
    fn parse_block_height(&self) -> Result<block::Height, Error> {
        block::Height::parse(self.height)
    }
}

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/remotesigner/SignedProposalResponse"]
pub struct SignedProposalResponse {
    #[prost(message, tag = "1")]
    pub proposal: Option<Proposal>,
    #[prost(message, tag = "2")]
    pub err: Option<RemoteError>,
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
            msg_type: SignedMsgType::Proposal.to_u32(),
            height: proposal.height,
            block_id: match proposal.block_id {
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
            pol_round: proposal.pol_round,
            round: proposal.round,
            timestamp: proposal.timestamp,
        };

        cp.encode_length_delimited(sign_bytes)?;
        Ok(true)
    }
    fn set_signature(&mut self, sig: &ed25519::Signature) {
        if let Some(ref mut prop) = self.proposal {
            prop.signature = sig.clone().into_vec();
        }
    }
    fn validate(&self) -> Result<(), ValidationError> {
        match self.proposal {
            Some(ref p) => p.validate_basic(),
            None => Err(ValidationError::new(
                ValidationErrorKind::MissingConsensusMessage,
            )),
        }
    }
}

impl ConsensusMessage for Proposal {
    fn validate_basic(&self) -> Result<(), ValidationError> {
        if self.msg_type != SignedMsgType::Proposal.to_u32() {
            return Err(ValidationError::new(
                ValidationErrorKind::InvalidMessageType,
            ));
        }
        if self.height < 0 {
            return Err(ValidationError::new(ValidationErrorKind::NegativeHeight));
        }
        if self.round < 0 {
            return Err(ValidationError::new(ValidationErrorKind::NegativeRound));
        }
        if self.pol_round < -1 {
            return Err(ValidationError::new(ValidationErrorKind::NegativePOLRound));
        }
        // TODO validate proposal's block_id

        // signature will be missing as the KMS provides it

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amino_types::block_id::PartsSetHeader;
    use chrono::{DateTime, Utc};
    use prost::Message;
    use std::error::Error;

    #[test]
    fn test_serialization() {
        let dt = "2018-02-11T07:09:22.765Z".parse::<DateTime<Utc>>().unwrap();
        let t = TimeMsg {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        };
        let proposal = Proposal {
            msg_type: SignedMsgType::Proposal.to_u32(),
            height: 12345,
            round: 23456,
            pol_round: -1,
            block_id: Some(BlockId {
                hash: "hash".as_bytes().to_vec(),
                parts_header: Some(PartsSetHeader {
                    total: 1000000,
                    hash: "parts_hash".as_bytes().to_vec(),
                }),
            }),
            timestamp: Some(t),
            signature: vec![],
        };
        let mut got = vec![];

        let _have = SignProposalRequest {
            proposal: Some(proposal),
        }.encode(&mut got);
        // test-vector generated via:
        // cdc := amino.NewCodec()
        // privval.RegisterRemoteSignerMsg(cdc)
        // stamp, _ := time.Parse(time.RFC3339Nano, "2018-02-11T07:09:22.765Z")
        // data, _ := cdc.MarshalBinaryLengthPrefixed(privval.SignProposalRequest{Proposal: &types.Proposal{
        //     Type:     types.ProposalType, // 0x20
        //     Height:   12345,
        //     Round:    23456,
        //     POLRound: -1,
        //     BlockID: types.BlockID{
        //         Hash: []byte("hash"),
        //         PartsHeader: types.PartSetHeader{
        //             Hash:  []byte("parts_hash"),
        //             Total: 1000000,
        //         },
        //     },
        //     Timestamp: stamp,
        // }})
        // fmt.Println(strings.Join(strings.Split(fmt.Sprintf("%v", data), " "), ", "))
        let want = vec![
            66, // len
            189, 228, 152, 226, // prefix
            10, 60, 8, 32, 16, 185, 96, 24, 160, 183, 1, 32, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 1, 42, 24, 10, 4, 104, 97, 115, 104, 18, 16, 8, 192, 132, 61, 18, 10, 112,
            97, 114, 116, 115, 95, 104, 97, 115, 104, 50, 12, 8, 162, 216, 255, 211, 5, 16, 192,
            242, 227, 236, 2,
        ];

        assert_eq!(got, want)
    }

    #[test]
    fn test_deserialization() {
        let dt = "2018-02-11T07:09:22.765Z".parse::<DateTime<Utc>>().unwrap();
        let t = TimeMsg {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        };
        let proposal = Proposal {
            msg_type: SignedMsgType::Proposal.to_u32(),
            height: 12345,
            round: 23456,
            timestamp: Some(t),

            pol_round: -1,
            block_id: Some(BlockId {
                hash: "hash".as_bytes().to_vec(),
                parts_header: Some(PartsSetHeader {
                    total: 1000000,
                    hash: "parts_hash".as_bytes().to_vec(),
                }),
            }),
            signature: vec![],
        };
        let want = SignProposalRequest {
            proposal: Some(proposal),
        };

        let data = vec![
            66, // len
            189, 228, 152, 226, // prefix
            10, 60, 8, 32, 16, 185, 96, 24, 160, 183, 1, 32, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 1, 42, 24, 10, 4, 104, 97, 115, 104, 18, 16, 8, 192, 132, 61, 18, 10, 112,
            97, 114, 116, 115, 95, 104, 97, 115, 104, 50, 12, 8, 162, 216, 255, 211, 5, 16, 192,
            242, 227, 236, 2,
        ];

        match SignProposalRequest::decode(&data) {
            Ok(have) => assert_eq!(have, want),
            Err(err) => assert!(false, err.description().to_string()),
        }
    }
}
