use chrono::{DateTime,Utc};
use signatory::ed25519::{Signature, SIGNATURE_SIZE};
use super::{ValidatorAddress,TendermintSign, BlockID};
use hex::encode_upper;

pub struct Vote{
    validator_address:ValidatorAddress,
    validator_index: i64,
    height: i64,
    round: i64,
    timestamp: DateTime<Utc>,
    vote_type:u8,
    block_id: BlockID,
    signature: Signature
}

impl TendermintSign for Vote{
    fn cannonicalize(self, chain_id: &str)->String{
        let value = json!({
            "@chain_id":chain_id,
            "@type":"vote",
            "block_id":{
                "hash":encode_upper(self.block_id.hash),
                "parts":{
                    "hash":encode_upper(self.block_id.parts_header.hash),
                    "total":self.block_id.parts_header.total
                }
            },
            "height":self.height,
            "round":self.round,
            "timestamp":self.timestamp.to_rfc3339(),
            "type":self.vote_type as char
            });
        value.to_string()
    }
}