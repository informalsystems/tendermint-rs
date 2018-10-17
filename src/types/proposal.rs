use super::{BlockID, PartsSetHeader, TendermintSign, Time};
use chrono::{DateTime, Utc};
use std::time::{SystemTime, UNIX_EPOCH};
use subtle_encoding::hex::encode_upper;

#[derive(Clone, PartialEq, Message)]
pub struct Proposal {
    #[prost(sint64, tag = "1")]
    height: i64,
    #[prost(sint64)]
    round: i64,
    #[prost(message)]
    timestamp: Option<Time>,
    #[prost(message)]
    block_parts_header: Option<PartsSetHeader>,
    #[prost(sint64)]
    pol_round: i64,
    #[prost(message)]
    pol_block_id: Option<BlockID>,
    #[prost(message)]
    signature: Option<Vec<u8>>,
}

pub const AMINO_NAME: &str = "tendermint/socketpv/SignProposalMsg";

#[derive(Clone, PartialEq, Message)]
#[amino_name = "tendermint/socketpv/SignProposalMsg"]
pub struct SignProposalMsg {
    #[prost(message, tag = "1")]
    proposal: Option<Proposal>,
}

impl TendermintSign for SignProposalMsg {
    fn cannonicalize(self, chain_id: &str) -> String {
        match self.proposal {
            Some(prop) => {
                let block_parts_header = {
                    match prop.block_parts_header {
                        Some(block_parts_header) => block_parts_header,
                        None => PartsSetHeader {
                            total: 0,
                            hash: vec![],
                        },
                    }
                };
                let pol_block_id = {
                    match prop.pol_block_id {
                        Some(pol_block_id) => pol_block_id,
                        None => BlockID {
                            hash: vec![],
                            parts_header: Some(PartsSetHeader {
                                total: 0,
                                hash: vec![],
                            }),
                        },
                    }
                };
                // this can not be None (see above)
                let pol_block_id_parts_header = pol_block_id.parts_header.unwrap();
                let ts: DateTime<Utc> = match prop.timestamp {
                    Some(timestamp) => DateTime::from(SystemTime::from(timestamp)),
                    None => DateTime::from(UNIX_EPOCH),
                };

                let value = json!({
            "@chain_id":chain_id,
            "@type":"proposal",
            "round":prop.round,
            "block_parts_header":{
                "hash":encode_upper(block_parts_header.hash),
                "total":block_parts_header.total
            },
            "height":prop.height,
            "pol_block_id":{
                "hash":encode_upper(pol_block_id.hash),
                "parts":{
                    "hash":encode_upper(pol_block_id_parts_header.hash),
                    "total":pol_block_id_parts_header.total,
                }
            },
            "pol_round":prop.pol_round,
            "round":prop.round,
            "timestamp": ts.to_rfc3339(),
            });
                value.to_string()
            }
            None => "".to_owned(),
        }
    }
    fn sign(&mut self) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::prost_amino::Message;
    use std::error::Error;

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

        let _have = SignProposalMsg {
            proposal: Some(proposal),
        }.encode(&mut got);
        let want = vec![
            0x31, 0x5d, 0x48, 0x70, 0x4, 0xa, 0x2b, 0x8, 0xf2, 0xc0, 0x1, 0x10, 0xc0, 0xee, 0x2,
            0x1a, 0xe, 0x9, 0x22, 0xec, 0x7f, 0x5a, 0x0, 0x0, 0x0, 0x0, 0x15, 0x40, 0xf9, 0x98,
            0x2d, 0x22, 0xf, 0x8, 0xde, 0x1, 0x12, 0xa, 0x62, 0x6c, 0x6f, 0x63, 0x6b, 0x70, 0x61,
            0x72, 0x74, 0x73, 0x28, 0x1,
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
        let want = SignProposalMsg {
            proposal: Some(proposal),
        };

        let data = vec![
            0x31, 0x5d, 0x48, 0x70, 0x4, 0xa, 0x2b, 0x8, 0xf2, 0xc0, 0x1, 0x10, 0xc0, 0xee, 0x2,
            0x1a, 0xe, 0x9, 0x22, 0xec, 0x7f, 0x5a, 0x0, 0x0, 0x0, 0x0, 0x15, 0x40, 0xf9, 0x98,
            0x2d, 0x22, 0xf, 0x8, 0xde, 0x1, 0x12, 0xa, 0x62, 0x6c, 0x6f, 0x63, 0x6b, 0x70, 0x61,
            0x72, 0x74, 0x73, 0x28, 0x1,
        ];

        match SignProposalMsg::decode(&data) {
            Ok(have) => assert_eq!(have, want),
            Err(err) => assert!(false, err.description().to_string()),
        }
    }
    // TODO Serialization with Signature should be fairly easy as the signature is just
    // an Option<bytes> now
}
