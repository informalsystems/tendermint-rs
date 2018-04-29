
use signatory::ed25519::Signature;
use super::{ValidatorAddress,TendermintSign};
use amino::*;
use hex::encode;
use bytes::BufMut;

pub struct Heartbeat{
    validator_address: ValidatorAddress,
    validator_index: i64,
    height: i64,
    round: i64,
    sequence:i64,
    signature:Option<Signature>,
}

impl TendermintSign for Heartbeat{
    fn cannonicalize(self, chain_id: &str)->String{
        let value = json!({
            "@chain_id":chain_id,
            "@type":"heartbeat",
            "height":self.height,
            "round":self.round,
            "sequence":self.sequence,
            "validator_address": encode(self.validator_address.0),
            "validator_index": self.validator_index,
            });
        value.to_string()
    }
}

impl Amino for Heartbeat{
    fn serialize(self)->Vec<u8>{
        let mut buf =  vec![];

        let  (_, mut pre) = compute_disfix("tendermint/socketpv/SignHeartbeatMsg");

        pre[3] |= typ3_to_byte(Typ3Byte::Typ3_Struct); 

        buf.put_slice(pre.as_slice());

//Encode the Validator Address
        {
            {
                let field_prefix = 1 << 3 | typ3_to_byte(Typ3Byte::Typ3_Struct); 
                buf.put(field_prefix);
            }
            {
                let field_prefix = 1 << 3 | typ3_to_byte(Typ3Byte::Typ3_ByteLength); 
                buf.put(field_prefix);
            }

            amino_bytes::encode(&self.validator_address.0, &mut buf);
        }
//Encode the validator index        
        {
            let field_prefix = 2 << 3 |typ3_to_byte(Typ3Byte::Typ3_Varint);
            buf.put(field_prefix);
            encode_varint(self.validator_index as i64, &mut buf);
        }
        {
            let field_prefix = 3 << 3 |typ3_to_byte(Typ3Byte::Typ3_8Byte);
            buf.put(field_prefix);
            encode_int64(self.height as u64, &mut buf);
        }
        {
            let field_prefix = 4 << 3 |typ3_to_byte(Typ3Byte::Typ3_Varint);
            buf.put(field_prefix);
            encode_varint(self.round as i64, &mut buf);
        }
        {
            let field_prefix = 5 << 3 |typ3_to_byte(Typ3Byte::Typ3_Varint);
            buf.put(field_prefix);
            encode_varint(self.sequence as i64, &mut buf);
        }

        println!("{:x?}",buf );

        // buf

        let mut length_buf = vec![];

        encode_uvarint(buf.len() as u64, &mut length_buf);

        length_buf.append(&mut buf);
        length_buf
    }

    fn deserialize(self, data: &[u8]){
        unimplemented!()
    }
}    

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let addr:[u8;20] =[0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35];
        let heartbeat = Heartbeat{ validator_address:ValidatorAddress(addr), validator_index:1, height: 15, round: 10, sequence: 30, signature:None };
        
        
        let have = heartbeat.serialize();

        let want = vec![0x2c, 0xbf, 0x58, 0xca, 0xeb, 0xb, 0xa, 0x14, 0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35, 0x10, 0x2, 0x19, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1e, 0x20, 0x14, 0x28, 0x1e, 0x4, 0x4];

        assert_eq!(have, want)
    }
}