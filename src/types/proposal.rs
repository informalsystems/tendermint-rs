use super::{BlockID, PartsSetHeader, TendermintSign};
use amino::*;
use bytes::{Buf, BufMut};
use chrono::{DateTime, Utc};
use hex::encode_upper;
use signatory::ed25519::{Signature, SIGNATURE_SIZE};
use std::io::Cursor;

#[derive(PartialEq, Debug)]
pub struct Proposal {
    height: i64,
    round: i64,
    timestamp: DateTime<Utc>,
    block_parts_header: PartsSetHeader,
    pol_round: i64,
    pol_block_id: BlockID,
    signature: Option<Signature>,
}

impl TendermintSign for Proposal {
    fn cannonicalize(self, chain_id: &str) -> String {
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

impl Amino for Proposal {
    fn serialize(self) -> Vec<u8> {
        let mut buf = vec![];
        let (_dis, mut pre) = compute_disfix("tendermint/socketpv/SignProposalMsg");

        pre[3] |= typ3_to_byte(Typ3Byte::Typ3_Struct);
        buf.put_slice(pre.as_slice());
        {
            encode_field_number_typ3(1, Typ3Byte::Typ3_Struct, &mut buf);
            {
                // height:
                encode_field_number_typ3(1, Typ3Byte::Typ3_8Byte, &mut buf);
                encode_int64(self.height as i64, &mut buf);
                // round:
                encode_field_number_typ3(2, Typ3Byte::Typ3_Varint, &mut buf);
                encode_varint(self.round as i64, &mut buf);

                // timestamp
                encode_field_number_typ3(3, Typ3Byte::Typ3_Struct, &mut buf);
                amino_time::encode(self.timestamp, &mut buf);

                // block parts header:
                encode_field_number_typ3(4, Typ3Byte::Typ3_Struct, &mut buf);
                {
                    encode_field_number_typ3(1, Typ3Byte::Typ3_Varint, &mut buf);
                    encode_varint(self.block_parts_header.total as i64, &mut buf);

                    if !&self.block_parts_header.hash.is_empty() {
                        encode_field_number_typ3(2, Typ3Byte::Typ3_ByteLength, &mut buf);
                        amino_bytes::encode(&self.block_parts_header.hash, &mut buf);
                    }
                }
                // end of block parts header struct
                buf.put(typ3_to_byte(Typ3Byte::Typ3_StructTerm));

                // Proof of Lock Round:
                encode_field_number_typ3(5, Typ3Byte::Typ3_Varint, &mut buf);
                encode_varint(self.pol_round as i64, &mut buf);

                // Proof of Lock (POL) block ID:
                encode_field_number_typ3(6, Typ3Byte::Typ3_Struct, &mut buf);
                {
                    // hash (encode only if not empty):
                    if !&self.pol_block_id.hash.is_empty() {
                        encode_field_number_typ3(1, Typ3Byte::Typ3_ByteLength, &mut buf);
                        amino_bytes::encode(&self.pol_block_id.hash, &mut buf);
                    }
                    // parts header:
                    encode_field_number_typ3(2, Typ3Byte::Typ3_Struct, &mut buf);
                    {
                        // always encode total; can't be empty (default is 0):
                        encode_field_number_typ3(1, Typ3Byte::Typ3_Varint, &mut buf);
                        encode_varint(self.pol_block_id.parts_header.total as i64, &mut buf);
                        // hash (encode only if not empty):
                        if !&self.pol_block_id.parts_header.hash.is_empty() {
                            encode_field_number_typ3(2, Typ3Byte::Typ3_ByteLength, &mut buf);
                            amino_bytes::encode(&self.pol_block_id.parts_header.hash, &mut buf);
                        }
                    }
                    // end of parts_header (in POL block ID)
                    buf.put(typ3_to_byte(Typ3Byte::Typ3_StructTerm));
                }
                // end of POL block ID
                buf.put(typ3_to_byte(Typ3Byte::Typ3_StructTerm));

                // Signature:
                if let Some(sig) = self.signature {
                    encode_field_number_typ3(7, Typ3Byte::Typ3_Interface, &mut buf);
                    amino_bytes::encode(&sig.0, &mut buf)
                }
            }
            // end of main struct
            buf.put(typ3_to_byte(Typ3Byte::Typ3_StructTerm));
        }
        // we are done here
        buf.put(typ3_to_byte(Typ3Byte::Typ3_StructTerm));

        let mut res = vec![];
        encode_uvarint(buf.len() as u64, &mut res);
        res.append(&mut buf);

        res
    }

    fn deserialize(data: &[u8]) -> Result<Proposal, DecodeError> {
        let mut buf = Cursor::new(data);
        consume_length(&mut buf)?;
        consume_prefix(&mut buf, "tendermint/socketpv/SignProposalMsg")?;
        check_field_number_typ3(1, Typ3Byte::Typ3_Struct, &mut buf)?;

        check_field_number_typ3(1, Typ3Byte::Typ3_8Byte, &mut buf)?;
        let height = decode_int64(&mut buf)?;

        check_field_number_typ3(2, Typ3Byte::Typ3_Varint, &mut buf)?;
        let round = decode_varint(&mut buf)?;

        check_field_number_typ3(3, Typ3Byte::Typ3_Struct, &mut buf)?;
        let timestamp = amino_time::decode(&mut buf)?;

        // field 4: PartSetHeader:
        check_field_number_typ3(4, Typ3Byte::Typ3_Struct, &mut buf)?;
        let parts_header_res = PartsSetHeader::decode(1, &mut buf);
        let block_parts_header = parts_header_res?;

        check_field_number_typ3(5, Typ3Byte::Typ3_Varint, &mut buf)?;
        let pol_round = decode_varint(&mut buf)? as i64;

        let block_id_res: Result<BlockID, DecodeError> = BlockID::decode(6, &mut buf);
        let pol_block_id = block_id_res?;

        let mut signature: Option<Signature> = None;
        let mut optional_typ3 = buf.get_u8();
        // TODO(ismail): find a more clever way to deal with optional fields:
        let sig_field_prefix = 6 << 3 | typ3_to_byte(Typ3Byte::Typ3_Interface);
        if optional_typ3 == sig_field_prefix {
            let mut signature_array: [u8; SIGNATURE_SIZE] = [0; SIGNATURE_SIZE];
            signature_array.copy_from_slice(amino_bytes::decode(&mut buf)?.as_slice());
            signature = Some(Signature(signature_array));

            optional_typ3 = buf.get_u8();
        }
        let _struct_term_typ3 = buf.get_u8();
        let struct_end_postfix = typ3_to_byte(Typ3Byte::Typ3_StructTerm);
        if optional_typ3 != struct_end_postfix {
            return Err(DecodeError::new("invalid type for first struct term"));
        }

        Ok(Proposal {
            height,
            round,
            timestamp,
            block_parts_header,
            pol_round,
            pol_block_id,
            signature,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_serialization() {
        let proposal = Proposal {
            height: 12345,
            round: 23456,
            timestamp: "2018-02-11T07:09:22.765Z".parse::<DateTime<Utc>>().unwrap(),
            block_parts_header: PartsSetHeader {
                total: 111,
                hash: "blockparts".as_bytes().to_vec(),
            },
            pol_round: -1,
            pol_block_id: BlockID {
                hash: vec![],
                parts_header: PartsSetHeader {
                    total: 0,
                    hash: vec![],
                },
            },
            signature: None,
        };

        let have = proposal.serialize();
        let want = vec![
            0x3d, 0x5d, 0x48, 0x70, 0x3, 0xb, 0x9, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x30, 0x39, 0x10,
            0xc0, 0xee, 0x2, 0x1b, 0x9, 0x0, 0x0, 0x0, 0x0, 0x5a, 0x7f, 0xec, 0x22, 0x15, 0x2d,
            0x98, 0xf9, 0x40, 0x4, 0x23, 0x8, 0xde, 0x1, 0x12, 0xa, 0x62, 0x6c, 0x6f, 0x63, 0x6b,
            0x70, 0x61, 0x72, 0x74, 0x73, 0x4, 0x28, 0x1, 0x33, 0x13, 0x8, 0x0, 0x4, 0x4, 0x4, 0x4,
        ];

        assert_eq!(have, want)
    }

    #[test]
    fn test_deserialization() {
        let want = Proposal {
            height: 12345,
            round: 23456,
            timestamp: "2018-02-11T07:09:22.765Z".parse::<DateTime<Utc>>().unwrap(),
            block_parts_header: PartsSetHeader {
                total: 111,
                hash: "blockparts".as_bytes().to_vec(),
            },
            pol_round: -1,
            pol_block_id: BlockID {
                hash: vec![],
                parts_header: PartsSetHeader {
                    total: 0,
                    hash: vec![],
                },
            },
            signature: None,
        };

        let data = vec![
            0x3d, 0x5d, 0x48, 0x70, 0x3, 0xb, 0x9, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x30, 0x39, 0x10,
            0xc0, 0xee, 0x2, 0x1b, 0x9, 0x0, 0x0, 0x0, 0x0, 0x5a, 0x7f, 0xec, 0x22, 0x15, 0x2d,
            0x98, 0xf9, 0x40, 0x4, 0x23, 0x8, 0xde, 0x1, 0x12, 0xa, 0x62, 0x6c, 0x6f, 0x63, 0x6b,
            0x70, 0x61, 0x72, 0x74, 0x73, 0x4, 0x28, 0x1, 0x33, 0x13, 0x8, 0x0, 0x4, 0x4, 0x4, 0x4,
        ];

        match Proposal::deserialize(&data) {
            Err(err) => assert!(false, err.description().to_string()),
            Ok(have) => assert_eq!(have, want),
        }
    }
    //ToDo Serialization with Signature
}
