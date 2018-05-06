use chrono::{DateTime,Utc};
use signatory::ed25519::{Signature, SIGNATURE_SIZE};
use super::{TendermintSign, BlockID, PartsSetHeader};
use hex::encode_upper;
use amino::*;



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

    // #[cfg(test)]
    // mod tests {
    //     use super::*;

    //     #[test]
    //     fn test_serialization() {
    //         let addr:[u8;20] =[0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35];
    //         let proposal = Proposal{ validator_address:ValidatorAddress(addr), validator_index:1, height: 15, round: 10, sequence: 30, signature:None };
            
            
    //         let have = proposal.serialize();

    //         let want = vec![0x2c, 0xbf, 0x58, 0xca, 0xeb, 0xb, 0xa, 0x14, 0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35, 0x10, 0x2, 0x19, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1e, 0x20, 0x14, 0x28, 0x1e, 0x4, 0x4];

    //         assert_eq!(have, want)
    //     }
    //     #[test]
    //     fn test_derialization(){
    //         let addr:[u8;20] =[0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35];
    //         let want = Proposal{ validator_address:ValidatorAddress(addr), validator_index:1, height: 15, round: 10, sequence: 30, signature:None };

    //         let data = vec![0x2c, 0xbf, 0x58, 0xca, 0xeb, 0xb, 0xa, 0x14, 0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35, 0x10, 0x2, 0x19, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1e, 0x20, 0x14, 0x28, 0x1e, 0x4, 0x4];

    //         match Proposal::deserialize(&data){
    //             Err(err) => assert!(false,err.description().to_string()),
    //             Ok(have) => assert_eq!(have,want)
    //         }


    //     }
    //     //ToDo Serialization with Signature
    // }