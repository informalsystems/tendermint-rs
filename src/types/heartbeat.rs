
use signatory::ed25519::Signature;
use super::{ValidatorAddress,TendermintSign};
use amino::*;
use hex::encode;
use bytes::{Buf,BufMut};
use std::error::Error;
use std::io::Cursor;


#[derive(PartialEq, Debug)]
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
            encode_int64(self.height as i64, &mut buf);
        }
        {
            let field_prefix = 4 << 3 |typ3_to_byte(Typ3Byte::Typ3_Varint);
            buf.put(field_prefix);
            encode_varint(self.round as i64, &mut buf);
        }
        {
            let field_prefix = 5 << 3 |typ3_to_byte(Typ3Byte::Typ3_Varint);
            buf.put(field_prefix);
            encode_uvarint(self.sequence as u64, &mut buf);
        }


        {
            if let Some(sig) = self.signature {
                let field_prefix = 6 <<3 | typ3_to_byte(Typ3Byte::Typ3_Interface);
                buf.put(field_prefix);
                amino_bytes::encode(&sig.0, &mut buf)
            }
        }


        {
            let struct_end_postfix = typ3_to_byte(Typ3Byte::Typ3_StructTerm);
            buf.put(struct_end_postfix);
            buf.put(struct_end_postfix);
        }


        let mut length_buf = vec![];

        encode_uvarint(buf.len() as u64, &mut length_buf);

        length_buf.append(&mut buf);

        println!("{:x?}",length_buf );
        length_buf
    }
}

pub fn deserialize_heartbeat(data: &[u8]) -> Result<Heartbeat,DecodeError> {
    
    
    let mut buf = Cursor::new(data);

    {
        let len_field = decode_uvarint(&mut buf)?;
        let data_length = buf.remaining() as u64;

        if data_length != len_field{
            return Err(DecodeError::new("invalid length field"));
        }
    }

    {

        let (_, mut pre) = compute_disfix("tendermint/socketpv/SignHeartbeatMsg");

        pre[3] |= typ3_to_byte(Typ3Byte::Typ3_Struct); 
        let mut actual_prefix =pre.clone();
        buf.copy_to_slice(actual_prefix.as_mut_slice());
        if actual_prefix != pre{
            return Err(DecodeError::new("invalid prefix"));
        }


    }
    {
        let typ3=buf.get_u8();
        let field_prefix = 1 << 3 | typ3_to_byte(Typ3Byte::Typ3_Struct); 
        if typ3 != field_prefix{
            return Err(DecodeError::new("invalid type for field 1"));
        }
    }
    {
        let typ3=buf.get_u8();
        let field_prefix = 1 << 3 | typ3_to_byte(Typ3Byte::Typ3_ByteLength); 
        if typ3 != field_prefix{
            return Err(DecodeError::new("invalid type for inner struct field 1"));
        }
    }
    let validator_address = amino_bytes::decode(&mut buf)?;

    unimplemented!()

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
    #[test]
    fn test_derialization(){
        let addr:[u8;20] =[0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35];
        let want = Heartbeat{ validator_address:ValidatorAddress(addr), validator_index:1, height: 15, round: 10, sequence: 30, signature:None };

        let data = vec![0x2c, 0xbf, 0x58, 0xca, 0xeb, 0xb, 0xa, 0x14, 0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35, 0x10, 0x2, 0x19, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1e, 0x20, 0x14, 0x28, 0x1e, 0x4, 0x4];

        match deserialize_heartbeat(&data){
            Err(err) => assert!(false,err.description().to_string()),
            Ok(have) => assert_eq!(have,want)
        }


    }

    //ToDo Serialization with Signature
}