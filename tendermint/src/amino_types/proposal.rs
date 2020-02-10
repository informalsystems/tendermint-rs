use super::{
    block_id::{BlockId, CanonicalBlockId, CanonicalPartSetHeader},
    compute_prefix,
    remote_error::RemoteError,
    signature::{SignableMsg, SignedMsgType},
    time::TimeMsg,
    validate::{ConsensusMessage, ValidationError, ValidationErrorKind::*},
};
use crate::{
    block::{self, ParseId},
    chain, consensus,
    error::Error,
};
use bytes::BufMut;
use once_cell::sync::Lazy;
use prost_amino::{EncodeError, Message};
use prost_amino_derive::Message;
use signatory::ed25519;
use std::convert::TryFrom;

#[derive(Clone, PartialEq, Message)]
pub struct Proposal {
    #[prost_amino(uint32, tag = "1")]
    pub msg_type: u32,
    #[prost_amino(int64)]
    pub height: i64,
    #[prost_amino(int64)]
    pub round: i64,
    #[prost_amino(int64)]
    pub pol_round: i64,
    #[prost_amino(message)]
    pub block_id: Option<BlockId>,
    #[prost_amino(message)]
    pub timestamp: Option<TimeMsg>,
    #[prost_amino(bytes)]
    pub signature: Vec<u8>,
}

// TODO(tony): custom derive proc macro for this e.g. `derive(ParseBlockHeight)`
impl block::ParseHeight for Proposal {
    fn parse_block_height(&self) -> Result<block::Height, Error> {
        block::Height::try_from(self.height)
    }
}

pub const AMINO_NAME: &str = "tendermint/remotesigner/SignProposalRequest";
pub static AMINO_PREFIX: Lazy<Vec<u8>> = Lazy::new(|| compute_prefix(AMINO_NAME));

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/remotesigner/SignProposalRequest"]
pub struct SignProposalRequest {
    #[prost_amino(message, tag = "1")]
    pub proposal: Option<Proposal>,
}

#[derive(Clone, PartialEq, Message)]
struct CanonicalProposal {
    #[prost_amino(uint32, tag = "1")]
    msg_type: u32, /* this is a byte in golang, which is a varint encoded UInt8 (using amino's
                    * EncodeUvarint) */
    #[prost_amino(sfixed64)]
    height: i64,
    #[prost_amino(sfixed64)]
    round: i64,
    #[prost_amino(sfixed64)]
    pol_round: i64,
    #[prost_amino(message)]
    block_id: Option<CanonicalBlockId>,
    #[prost_amino(message)]
    timestamp: Option<TimeMsg>,
    #[prost_amino(string)]
    pub chain_id: String,
}

impl chain::ParseId for CanonicalProposal {
    fn parse_chain_id(&self) -> Result<chain::Id, Error> {
        self.chain_id.parse()
    }
}

impl block::ParseHeight for CanonicalProposal {
    fn parse_block_height(&self) -> Result<block::Height, Error> {
        block::Height::try_from(self.height)
    }
}

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/remotesigner/SignedProposalResponse"]
pub struct SignedProposalResponse {
    #[prost_amino(message, tag = "1")]
    pub proposal: Option<Proposal>,
    #[prost_amino(message, tag = "2")]
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
            prop.signature = sig.as_ref().to_vec();
        }
    }
    fn validate(&self) -> Result<(), ValidationError> {
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
                round: p.round,
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
    fn validate_basic(&self) -> Result<(), ValidationError> {
        if self.msg_type != SignedMsgType::Proposal.to_u32() {
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
    use crate::amino_types::block_id::PartsSetHeader;
    use chrono::{DateTime, Utc};
    use prost_amino::Message;

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
                hash: b"hash".to_vec(),
                parts_header: Some(PartsSetHeader {
                    total: 1_000_000,
                    hash: b"parts_hash".to_vec(),
                }),
            }),
            timestamp: Some(t),
            signature: vec![],
        };
        let mut got = vec![];

        let _have = SignProposalRequest {
            proposal: Some(proposal),
        }
        .encode(&mut got);
        // test-vector generated via:
        // cdc := amino.NewCodec()
        // privval.RegisterRemoteSignerMsg(cdc)
        // stamp, _ := time.Parse(time.RFC3339Nano, "2018-02-11T07:09:22.765Z")
        // data, _ := cdc.MarshalBinaryLengthPrefixed(privval.SignProposalRequest{Proposal:
        // &types.Proposal{     Type:     types.ProposalType, // 0x20
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
                hash: b"hash".to_vec(),
                parts_header: Some(PartsSetHeader {
                    total: 1_000_000,
                    hash: b"parts_hash".to_vec(),
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

        match SignProposalRequest::decode(data.as_ref()) {
            Ok(have) => assert_eq!(have, want),
            Err(err) => panic!(err.to_string()),
        }
    }
}
