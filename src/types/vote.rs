use super::{BlockID, TendermintSign, Time};
use bytes::{Buf, BufMut};
use chrono::{DateTime, Utc};
use hex::encode_upper;
use std::io::Cursor;
// TODO(ismail): we might not want to use this error type here
// see below: those aren't prost errors
use prost::error::DecodeError;
use prost::Message;

enum VoteType {
    PreVote,
    PreCommit,
}

fn vote_type_to_char(vt: VoteType) -> char {
    match vt {
        VoteType::PreVote => 0x01 as char,
        VoteType::PreCommit => 0x02 as char,
    }
}

fn char_to_vote_type(data: u32) -> Result<VoteType, DecodeError> {
    match data {
        1 => Ok(VoteType::PreVote),
        2 => Ok(VoteType::PreCommit),
        _ => Err(DecodeError::new("Invalid vote type")),
    }
}


#[derive(Clone, PartialEq, Message)]
pub struct Vote {
    #[prost(bytes, tag="1")]
    validator_address: Vec<u8>,
    #[prost(sint64)]
    validator_index: i64,
    #[prost(sint64)]
    height: i64,
    #[prost(sint64)]
    round: i64,
    #[prost(message)]
    timestamp: Option<Time>,
    #[prost(uint32)]
    vote_type: u32,
    #[prost(message)]
    block_id: Option<BlockID>,
    #[prost(message)]
    signature: Option<Vec<u8>>,
}

//impl TendermintSign for Vote {
//    fn cannonicalize(self, chain_id: &str) -> String {
//        let value = json!({
//            "@chain_id":chain_id,
//            "@type":"vote",
//            "block_id":{
//                "hash":encode_upper(self.block_id.hash),
//                "parts":{
//                    "hash":encode_upper(self.block_id.parts_header.hash),
//                    "total":self.block_id.parts_header.total
//                }
//            },
//            "height":self.height,
//            "round":self.round,
//            "timestamp":self.timestamp.to_rfc3339(),
//            "type":vote_type_to_char(self.vote_type)
//            });
//        value.to_string()
//    }
//}

#[cfg(test)]
mod tests {
    use super::*;
    //use types::PartsSetHeader;

    #[test]
    fn test_vote_serialization() {
        // TODO
    }

    #[test]
    fn test_derialization() {
        // TODO
    }
}
