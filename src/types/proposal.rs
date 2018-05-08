use chrono::{DateTime,Utc};
use signatory::ed25519::{Signature, SIGNATURE_SIZE};
use super::{TendermintSign, BlockID, PartsSetHeader};
use hex::{encode_upper,encode};
use amino::*;
use bytes::{Buf,BufMut};



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

impl Amino for Proposal{
        fn serialize(self)->Vec<u8>{
            unimplemented!()
        }
        fn deserialize(data: &[u8])->Result<Proposal,DecodeError>{
            unimplemented!()
        }
}

    #[cfg(test)]
    mod tests {
        use super::*;
        use chrono::prelude::*;
        use std::error::Error;
        #[test]
        fn test_serialization() {
            let proposal = Proposal{ 
                height: 12345, 
                round: 23456, 
                timestamp:"2018-02-11T07:09:22.765Z".parse::<DateTime<Utc>>().unwrap(),
                block_parts_header:PartsSetHeader{
                    total:111,
                    hash:"parts_hash".as_bytes().to_vec()
                },
                pol_round:-1,
                pol_block_id: BlockID{
                    hash: vec![],
                    parts_header:PartsSetHeader{
                        total:0,
                        hash:vec![]
                    },
                },
                signature:None 
                };
            
            
            let have = proposal.serialize();

            let want = vec![0x3d, 0x5d, 0x48, 0x70, 0x3, 0xb, 0x9, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x30, 0x39, 0x10, 0xc0, 0xee, 0x2, 0x1b, 0x9, 0x0, 0x0, 0x0, 0x0, 0x5a, 0x7f, 0xec, 0x22, 0x15, 0x2d, 0x98, 0xf9, 0x40, 0x4, 0x23, 0x8, 0xde, 0x1, 0x12, 0xa, 0x62, 0x6c, 0x6f, 0x63, 0x6b, 0x70, 0x61, 0x72, 0x74, 0x73, 0x4, 0x28, 0x1, 0x33, 0x13, 0x8, 0x0, 0x4, 0x4, 0x4, 0x4];

            assert_eq!(have, want)
        }
        #[test]
        fn test_deserialization(){
        let want = Proposal{ 
            height: 12345, 
            round: 23456, 
            timestamp:"2018-02-11T07:09:22.765Z".parse::<DateTime<Utc>>().unwrap(),
            block_parts_header:PartsSetHeader{
                total:111,
                hash:"parts_hash".as_bytes().to_vec()
            },
            pol_round:-1,
            pol_block_id: BlockID{
                hash: vec![],
                parts_header:PartsSetHeader{
                    total:0,
                hash:vec![]
                },
            },
            signature:None 
            };
        
            let data = vec![0x3d, 0x5d, 0x48, 0x70, 0x3, 0xb, 0x9, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x30, 0x39, 0x10, 0xc0, 0xee, 0x2, 0x1b, 0x9, 0x0, 0x0, 0x0, 0x0, 0x5a, 0x7f, 0xec, 0x22, 0x15, 0x2d, 0x98, 0xf9, 0x40, 0x4, 0x23, 0x8, 0xde, 0x1, 0x12, 0xa, 0x62, 0x6c, 0x6f, 0x63, 0x6b, 0x70, 0x61, 0x72, 0x74, 0x73, 0x4, 0x28, 0x1, 0x33, 0x13, 0x8, 0x0, 0x4, 0x4, 0x4, 0x4];

            match Proposal::deserialize(&data){
                Err(err) => assert!(false,err.description().to_string()),
                Ok(have) => assert_eq!(have,want)
            }


        }
        //ToDo Serialization with Signature
    }
