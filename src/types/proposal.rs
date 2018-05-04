use chrono::{DateTime,Utc};
use signatory::ed25519::{Signature, SIGNATURE_SIZE};
use super::{TendermintSign, BlockID};
use hex::encode_upper;


#[derive(PartialEq, Debug)]
struct PartsSetHeader{
    total: i64,
    hash: Vec<u8>
}


#[derive(PartialEq, Debug)]
pub struct Proposal{
    height:i64,
    round: i64,
    timestamp: DateTime<Utc>,
    block_parts_header: PartsSetHeader,
    pol_round:i64,
    pol_block_id: BlockID,
    signature:Option<Signature>,
}

impl TendermintSign for Proposal{
    fn cannonicalize(self, chain_id: &str)->String{
        let value = json!({
            "@chain_id":chain_id,
            "@type":"proposal",
            "round":self.round,
            "block_parts_header":{
                "hash":encode_upper(self.block_parts_header.hash),
                "total":self.block_parts_header.total
            },
            "height":self.height,
            "pol_block_id":{
                "hash":encode_upper(self.pol_block_id.hash),
                "parts":{
                    "hash":encode_upper(self.pol_block_id.parts_header.hash),
                    "total":self.pol_block_id.parts_header.total
                }
            },
            "pol_round":self.pol_round,
            "round":self.round,
            "timestamp":self.timestamp.to_rfc3339() 
            });
        value.to_string()
    }
}
