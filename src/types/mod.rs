use amino::*;
use bytes::Buf;
use std::io::Cursor;

mod heartbeat;
mod proposal;
mod vote;

pub type ValidatorAddress = Vec<u8>;

#[derive(PartialEq, Debug)]
pub struct PartsSetHeader {
    total: i64,
    hash: Vec<u8>,
}

impl PartsSetHeader {
    // decode expects the field number and type to already be consumed:
    fn decode(
        expected_field_num: u32,
        mut buf: &mut Cursor<&[u8]>,
    ) -> Result<PartsSetHeader, DecodeError> {
        check_field_number_typ3(expected_field_num, Typ3Byte::Typ3_Varint, &mut buf)?;
        let total = decode_varint(&mut buf)?;
        // peek into the buffer without consuming it and only read if necessary:
        let next_u8 = buf.bytes()[0];
        let hash: Vec<u8> = if ((next_u8 as u64) >> 3) == 2
            && byte_to_type3(next_u8 & 0x07) == Typ3Byte::Typ3_ByteLength
        {
            buf.advance(1);
            amino_bytes::decode(&mut buf)?
        } else {
            Vec::new()
        };
        let next_u8 = buf.get_u8();
        if next_u8 != typ3_to_byte(Typ3Byte::Typ3_StructTerm) {
            return Err(DecodeError::new(format!(
                "could not find struct term for field {}",
                expected_field_num
            )));
        }
        Ok(PartsSetHeader { total, hash })
    }
}

#[derive(PartialEq, Debug)]
struct BlockID {
    hash: Vec<u8>,
    parts_header: PartsSetHeader,
}

impl BlockID {
    fn decode(
        expected_field_num: u32,
        mut buf: &mut Cursor<&[u8]>,
    ) -> Result<BlockID, DecodeError> {
        check_field_number_typ3(expected_field_num, Typ3Byte::Typ3_Struct, &mut buf)?;
        // see what we get next hash (1) or embedded struct 2
        let mut hash: Vec<u8> = Vec::new();
        let (field_number, typ3) = decode_field_number_typ3(&mut buf)?;
        if field_number == 1 && typ3 == Typ3Byte::Typ3_ByteLength {
            hash = amino_bytes::decode(&mut buf)?;
        } else if field_number == 2 && typ3 == Typ3Byte::Typ3_Struct {
            // we are OK and we continue decoding partset_header below ...
        } else {
            return Err(DecodeError::new("Write sth. useful here"));
        }
        // if the first field was not empty:
        if field_number == 1 {
            check_field_number_typ3(2, Typ3Byte::Typ3_Struct, &mut buf)?;
        }
        let parts_header_res = PartsSetHeader::decode(1, &mut buf);
        let parts_header = parts_header_res?;
        let next_u8 = buf.get_u8();
        if next_u8 != typ3_to_byte(Typ3Byte::Typ3_StructTerm) {
            return Err(DecodeError::new("could not find struct term for BlockID"));
        }
        Ok(BlockID { hash, parts_header })
    }
}

pub trait TendermintSign {
    fn cannonicalize(self, chain_id: &str) -> String;
}

pub use self::heartbeat::Heartbeat;
