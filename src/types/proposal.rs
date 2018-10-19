use super::{
    BlockID, CanonicalBlockID, CanonicalPartSetHeader, Ed25519Signature, PartsSetHeader,
    RemoteError, Signature, SignedMsgType, TendermintSignable, Time,
};
use bytes::BufMut;
use prost::{EncodeError, Message};

#[derive(Clone, PartialEq, Message)]
pub struct Proposal {
    #[prost(sint64, tag = "1")]
    pub height: i64,
    #[prost(sint64)]
    pub round: i64,
    #[prost(message)]
    pub timestamp: Option<Time>,
    #[prost(message)]
    pub block_parts_header: Option<PartsSetHeader>,
    #[prost(sint64)]
    pub pol_round: i64,
    #[prost(message)]
    pub pol_block_id: Option<BlockID>,
    #[prost(message)]
    pub signature: Option<Vec<u8>>,
}

pub const AMINO_NAME: &str = "tendermint/remotesigner/SignProposalRequest";

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/remotesigner/SignProposalRequest"]
pub struct SignProposalRequest {
    #[prost(message, tag = "1")]
    pub proposal: Option<Proposal>,
}

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/remotesigner/SignedProposalResponse"]
pub struct SignedProposalResponse {
    #[prost(message, tag = "1")]
    pub proposal: Option<Proposal>,
    #[prost(message, tag = "2")]
    pub err: Option<RemoteError>,
}

#[derive(Clone, PartialEq, Message)]
struct CanonicalProposal {
    #[prost(sint64, tag = "1")]
    pub height: i64,
    #[prost(sint64)]
    round: i64,
    #[prost(uint32)]
    pub msg_type: u32, // this is a byte in golang, which is a varint encoded UInt8 (using amino's EncodeUvarint)

    #[prost(message)]
    timestamp: Option<Time>,
    #[prost(message)]
    block_parts_header: Option<CanonicalPartSetHeader>,
    #[prost(sint64)]
    pol_round: i64,
    #[prost(message)]
    pol_block_id: Option<CanonicalBlockID>,
    #[prost(string)]
    pub chain_id: String,
}

impl TendermintSignable for SignProposalRequest {
    fn sign_bytes<B>(&self, chain_id: &str, sign_bytes: &mut B) -> Result<bool, EncodeError>
    where
        B: BufMut,
    {
        let mut spr = self.clone();
        if let Some(ref mut pr) = spr.proposal {
            pr.signature = None
        }
        let proposal = spr.proposal.unwrap();
        let cp = CanonicalProposal {
            chain_id: chain_id.to_string(),
            msg_type: SignedMsgType::Proposal.to_u32(),
            block_parts_header: match proposal.block_parts_header {
                Some(ph) => Some(CanonicalPartSetHeader {
                    hash: ph.hash,
                    total: ph.total,
                }),
                None => None,
            },
            height: proposal.height,
            pol_block_id: match proposal.pol_block_id {
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
            pol_round: proposal.pol_round,
            round: proposal.round,
            timestamp: proposal.timestamp,
        };

        cp.encode(sign_bytes)?;
        Ok(true)
    }
    fn set_signature(&mut self, sig: &Ed25519Signature) {
        if let Some(ref mut prop) = self.proposal {
            prop.signature = Some(sig.clone().into_vec());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use std::error::Error;
    use types::prost_amino::Message;

    #[test]
    fn test_serialization() {
        let dt = "2018-02-11T07:09:22.765Z".parse::<DateTime<Utc>>().unwrap();
        let t = Time {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        };
        let proposal = Proposal {
            height: 12345,
            round: 23456,
            timestamp: Some(t),
            block_parts_header: Some(PartsSetHeader {
                total: 111,
                hash: "blockparts".as_bytes().to_vec(),
            }),
            pol_round: -1,
            pol_block_id: None,
            signature: None,
        };
        let mut got = vec![];

        let _have = SignProposalRequest {
            proposal: Some(proposal),
        }.encode(&mut got);
        let want = vec![
            0x31, // len
            189, 228, 152, 226, // prefix
            0xa, 0x2b, 0x8, 0xf2, 0xc0, 0x1, 0x10, 0xc0, 0xee, 0x2, 0x1a, 0xe, 0x9, 0x22, 0xec,
            0x7f, 0x5a, 0x0, 0x0, 0x0, 0x0, 0x15, 0x40, 0xf9, 0x98, 0x2d, 0x22, 0xf, 0x8, 0xde,
            0x1, 0x12, 0xa, 0x62, 0x6c, 0x6f, 0x63, 0x6b, 0x70, 0x61, 0x72, 0x74, 0x73, 0x28, 0x1,
        ];

        assert_eq!(got, want)
    }

    #[test]
    fn test_deserialization() {
        let dt = "2018-02-11T07:09:22.765Z".parse::<DateTime<Utc>>().unwrap();
        let t = Time {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        };
        let proposal = Proposal {
            height: 12345,
            round: 23456,
            timestamp: Some(t),
            block_parts_header: Some(PartsSetHeader {
                total: 111,
                hash: "blockparts".as_bytes().to_vec(),
            }),
            pol_round: -1,
            pol_block_id: None,
            signature: None,
        };
        let want = SignProposalRequest {
            proposal: Some(proposal),
        };

        let data = vec![
            0x31, 189, 228, 152, 226, 0xa, 0x2b, 0x8, 0xf2, 0xc0, 0x1, 0x10, 0xc0, 0xee, 0x2, 0x1a,
            0xe, 0x9, 0x22, 0xec, 0x7f, 0x5a, 0x0, 0x0, 0x0, 0x0, 0x15, 0x40, 0xf9, 0x98, 0x2d,
            0x22, 0xf, 0x8, 0xde, 0x1, 0x12, 0xa, 0x62, 0x6c, 0x6f, 0x63, 0x6b, 0x70, 0x61, 0x72,
            0x74, 0x73, 0x28, 0x1,
        ];

        match SignProposalRequest::decode(&data) {
            Ok(have) => assert_eq!(have, want),
            Err(err) => assert!(false, err.description().to_string()),
        }
    }
}
