use super::{BlockID, PartsSetHeader, TendermintSign, ValidatorAddress};
use amino::*;
use bytes::{Buf, BufMut};
use chrono::{DateTime, Utc};
use hex::{encode, encode_upper};
use signatory::ed25519::{Signature, SIGNATURE_SIZE};
use std::io::Cursor;

#[derive(PartialEq, Debug)]
enum VoteType {
    PreVote,
    PreCommit,
}

fn vote_type_to_char(vt: VoteType) -> char {
    match vt {
        pre_vote => 0x01 as char,
        pre_commit => 0x02 as char,
    }
}

fn char_to_vote_type(data: char) -> Result<VoteType, DecodeError> {
    let pre_vote = 0x01 as char;
    let pre_commit = 0x02 as char;

    match data {
        pre_vote => Ok(VoteType::PreVote),
        pre_commit => Ok(VoteType::PreCommit),
        _ => Err(DecodeError::new("Invalid vote type")),
    }
}

#[derive(PartialEq, Debug)]
pub struct Vote {
    validator_address: ValidatorAddress,
    validator_index: i64,
    height: i64,
    round: i64,
    timestamp: DateTime<Utc>,
    vote_type: VoteType,
    block_id: BlockID,
    signature: Option<Signature>,
}

impl TendermintSign for Vote {
    fn cannonicalize(self, chain_id: &str) -> String {
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
            "type":vote_type_to_char(self.vote_type)
            });
        value.to_string()
    }
}

impl Amino for Vote {
    fn serialize(self) -> Vec<u8> {
        let mut buf = vec![];
        let (dis, mut pre) = compute_disfix("tendermint/socketpv/SignVoteMsg");

        pre[3] |= typ3_to_byte(Typ3Byte::Typ3_Struct);
        buf.put_slice(pre.as_slice());
        {
            encode_field_number_typ3(1, Typ3Byte::Typ3_Struct, &mut buf);
            {
                //Encode the Validator Address
                encode_field_number_typ3(1, Typ3Byte::Typ3_ByteLength, &mut buf);
                amino_bytes::encode(&self.validator_address, &mut buf);

                //Encode the validator index
                encode_field_number_typ3(2, Typ3Byte::Typ3_Varint, &mut buf);
                encode_varint(self.validator_index as i64, &mut buf);

                //Encode the validator height
                encode_field_number_typ3(3, Typ3Byte::Typ3_8Byte, &mut buf);
                encode_int64(self.height as i64, &mut buf);

                encode_field_number_typ3(4, Typ3Byte::Typ3_Varint, &mut buf);
                encode_varint(self.round as i64, &mut buf);

                encode_field_number_typ3(5, Typ3Byte::Typ3_Struct, &mut buf);
                amino_time::encode(self.timestamp, &mut buf);
                // amino_time::encode takes care of Typ3_StructTerm

                encode_field_number_typ3(6, Typ3Byte::Typ3_Varint, &mut buf);
                encode_uint8(vote_type_to_char(self.vote_type) as u8, &mut buf);

                // Encode BlockID (struct)
                encode_field_number_typ3(7, Typ3Byte::Typ3_Struct, &mut buf);
                {
                    if !&self.block_id.hash.is_empty() {
                        encode_field_number_typ3(1, Typ3Byte::Typ3_ByteLength, &mut buf);
                        amino_bytes::encode(&self.block_id.hash, &mut buf);
                    }

                    encode_field_number_typ3(2, Typ3Byte::Typ3_Struct, &mut buf);
                    {
                        encode_field_number_typ3(1, Typ3Byte::Typ3_Varint, &mut buf);
                        encode_varint(self.block_id.parts_header.total, &mut buf);

                        if !&self.block_id.parts_header.hash.is_empty() {
                            encode_field_number_typ3(2, Typ3Byte::Typ3_ByteLength, &mut buf);
                            amino_bytes::encode(&self.block_id.parts_header.hash, &mut buf)
                        }
                    }
                    // end of embedded PartsSetHeader struct
                    buf.put(typ3_to_byte(Typ3Byte::Typ3_StructTerm));
                }
                // end of embedded BlockID struct
                buf.put(typ3_to_byte(Typ3Byte::Typ3_StructTerm));

                // Encode Signature:
                if let Some(sig) = self.signature {
                    encode_field_number_typ3(8, Typ3Byte::Typ3_Interface, &mut buf);
                    amino_bytes::encode(&sig.0, &mut buf)
                }
            }
            // signal end of main struct
            buf.put(typ3_to_byte(Typ3Byte::Typ3_StructTerm));
        }
        // we are done here ...
        buf.put(typ3_to_byte(Typ3Byte::Typ3_StructTerm));

        let mut length_buf = vec![];
        encode_uvarint(buf.len() as u64, &mut length_buf);
        length_buf.append(&mut buf);

        length_buf
    }

    fn deserialize(data: &[u8]) -> Result<Vote, DecodeError> {
        let mut buf = Cursor::new(data);

        {
            let len_field = decode_uvarint(&mut buf)?;
            let data_length = buf.remaining() as u64;

            if data_length != len_field {
                return Err(DecodeError::new("invalid length field"));
            }
        }

        {
            let (_, mut pre) = compute_disfix("tendermint/socketpv/SignVoteMsg");

            pre[3] |= typ3_to_byte(Typ3Byte::Typ3_Struct);
            let mut actual_prefix = pre.clone();
            buf.copy_to_slice(actual_prefix.as_mut_slice());
            if actual_prefix != pre {
                return Err(DecodeError::new("invalid prefix"));
            }
        }

        check_field_number_typ3(1, Typ3Byte::Typ3_Struct,&mut buf)?;
        check_field_number_typ3( 1, Typ3Byte::Typ3_ByteLength,&mut buf)?;
        let mut validator_address_array: [u8; 20] = [0; 20];
        validator_address_array.copy_from_slice(amino_bytes::decode(&mut buf)?.as_slice());
        let validator_address = validator_address_array;
        check_field_number_typ3( 2, Typ3Byte::Typ3_Varint, &mut buf)?;
        let validator_index = decode_varint(&mut buf)? as i64;
        check_field_number_typ3(3,Typ3Byte::Typ3_8Byte,&mut buf,)?;
        let height = decode_int64(&mut buf)?;
        check_field_number_typ3(4, Typ3Byte::Typ3_Varint,&mut buf)?;
        let round = decode_varint(&mut buf)?;
        check_field_number_typ3( 5, Typ3Byte::Typ3_Struct,&mut buf)?;
        let timestamp = amino_time::decode(&mut buf)?;
        check_field_number_typ3(6, Typ3Byte::Typ3_Varint,&mut buf)?;
        let vote_type = char_to_vote_type(decode_uint8(&mut buf)? as char)?;
        check_field_number_typ3(7, Typ3Byte::Typ3_Struct,&mut buf)?;
        
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;
    use std::error::Error;

    #[test]
    fn test_vote_serialization() {
        let addr: [u8; 20] = [
            0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
            0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
        ];
        {
            let vote = Vote {
                validator_address: addr,
                validator_index: 56789,
                height: 12345,
                round: 2,
                block_id: BlockID {
                    hash: "hash".as_bytes().to_vec(),
                    parts_header: PartsSetHeader {
                        total: 1000000,
                        hash: "parts_hash".as_bytes().to_vec(),
                    },
                },
                timestamp: "2017-12-25T03:00:01.234Z".parse::<DateTime<Utc>>().unwrap(),
                vote_type: VoteType::PreVote,
                signature: None,
            };

            let have = vote.serialize();
            let want = vec![
                0x58, 0x6c, 0x1d, 0x3a, 0x33, 0xb, 0xa, 0x14, 0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86,
                0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
                0x10, 0xaa, 0xf7, 0x6, 0x19, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x30, 0x39, 0x20, 0x4,
                0x2b, 0x9, 0x0, 0x0, 0x0, 0x0, 0x5a, 0x40, 0x69, 0xb1, 0x15, 0xd, 0xf2, 0x8e, 0x80,
                0x4, 0x30, 0x1, 0x3b, 0xa, 0x4, 0x68, 0x61, 0x73, 0x68, 0x13, 0x8, 0x80, 0x89,
                0x7a, 0x12, 0xa, 0x70, 0x61, 0x72, 0x74, 0x73, 0x5f, 0x68, 0x61, 0x73, 0x68, 0x4,
                0x4, 0x4, 0x4,
            ];
            assert_eq!(have, want)
        }
        {
            let vote = Vote {
                validator_address: addr,
                validator_index: 56789,
                height: 12345,
                round: 2,
                block_id: BlockID {
                    hash: "hash".as_bytes().to_vec(),
                    parts_header: PartsSetHeader {
                        total: 0,
                        hash: vec![],
                    },
                },
                timestamp: "2017-12-25T03:00:01.234Z".parse::<DateTime<Utc>>().unwrap(),
                vote_type: VoteType::PreVote,
                signature: None,
            };

            let have = vote.serialize();
            let want = vec![
                0x4a, 0x6c, 0x1d, 0x3a, 0x33, 0xb, 0xa, 0x14, 0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86,
                0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
                0x10, 0xaa, 0xf7, 0x6, 0x19, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x30, 0x39, 0x20, 0x4,
                0x2b, 0x9, 0x0, 0x0, 0x0, 0x0, 0x5a, 0x40, 0x69, 0xb1, 0x15, 0xd, 0xf2, 0x8e, 0x80,
                0x4, 0x30, 0x1, 0x3b, 0xa, 0x4, 0x68, 0x61, 0x73, 0x68, 0x13, 0x8, 0x0, 0x4, 0x4,
                0x4, 0x4,
            ];
            assert_eq!(have, want)
        }
    }

    #[test]
    fn test_derialization() {
        let addr: [u8; 20] = [
            0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1, 0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4,
            0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35,
        ];
        let want = Vote {
            validator_address: addr,
            validator_index: 56789,
            height: 12345,
            round: 2,
            block_id: BlockID {
                hash: "hash".as_bytes().to_vec(),
                parts_header: PartsSetHeader {
                    total: 1000000,
                    hash: "parts_hash".as_bytes().to_vec(),
                },
            },
            timestamp: "2017-12-25T03:00:01.234Z".parse::<DateTime<Utc>>().unwrap(),
            vote_type: VoteType::PreVote,
            signature: None,
        };
        let data = vec![
            0x58, 0x6c, 0x1d, 0x3a, 0x33, 0xb, 0xa, 0x14, 0xa3, 0xb2, 0xcc, 0xdd, 0x71, 0x86, 0xf1,
            0x68, 0x5f, 0x21, 0xf2, 0x48, 0x2a, 0xf4, 0xfb, 0x34, 0x46, 0xa8, 0x4b, 0x35, 0x10,
            0xaa, 0xf7, 0x6, 0x19, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x30, 0x39, 0x20, 0x4, 0x2b, 0x9,
            0x0, 0x0, 0x0, 0x0, 0x5a, 0x40, 0x69, 0xb1, 0x15, 0xd, 0xf2, 0x8e, 0x80, 0x4, 0x30,
            0x1, 0x3b, 0xa, 0x4, 0x68, 0x61, 0x73, 0x68, 0x13, 0x8, 0x80, 0x89, 0x7a, 0x12, 0xa,
            0x70, 0x61, 0x72, 0x74, 0x73, 0x5f, 0x68, 0x61, 0x73, 0x68, 0x4, 0x4, 0x4, 0x4,
        ];

        match Vote::deserialize(&data) {
            Err(err) => assert!(false, err.description().to_string()),
            Ok(have) => assert_eq!(have, want),
        }
    }
    //ToDo Serialization with Signature
}
