use crate::prelude::*;
use crate::{
    block::parts::Header as PartSetHeader,
    error::Error,
    hash::{Algorithm, Hash},
};
use core::convert::{TryFrom, TryInto};
use core::{
    fmt::{self, Display},
    str::{self, FromStr},
};
use serde::{Deserialize, Serialize};
use tendermint_proto::types::{
    BlockId as RawBlockId, CanonicalBlockId as RawCanonicalBlockId,
    PartSetHeader as RawPartSetHeader,
};
use tendermint_proto::Protobuf;

/// Length of a block ID prefix displayed for debugging purposes
pub const PREFIX_LENGTH: usize = 10;

/// Block identifiers which contain two distinct Merkle roots of the block,
/// as well as the number of parts in the block.
///
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#blockid>
///
/// Default implementation is an empty Id as defined by the Go implementation in
/// <https://github.com/tendermint/tendermint/blob/1635d1339c73ae6a82e062cd2dc7191b029efa14/types/block.go#L1204>.
///
/// If the Hash is empty in BlockId, the BlockId should be empty (encoded to None).
/// This is implemented outside of this struct. Use the Default trait to check for an empty BlockId.
/// See: <https://github.com/informalsystems/tendermint-rs/issues/663>
#[derive(
    Serialize, Deserialize, Copy, Clone, Debug, Default, Hash, Eq, PartialEq, PartialOrd, Ord,
)]
#[serde(try_from = "RawBlockId", into = "RawBlockId")]
pub struct Id {
    /// The block's main hash is the Merkle root of all the fields in the
    /// block header.
    pub hash: Hash,

    /// Parts header (if available) is used for secure gossipping of the block
    /// during consensus. It is the Merkle root of the complete serialized block
    /// cut into parts.
    ///
    /// PartSet is used to split a byteslice of data into parts (pieces) for
    /// transmission. By splitting data into smaller parts and computing a
    /// Merkle root hash on the list, you can verify that a part is
    /// legitimately part of the complete data, and the part can be forwarded
    /// to other peers before all the parts are known. In short, it's a fast
    /// way to propagate a large file over a gossip network.
    ///
    /// <https://github.com/tendermint/tendermint/wiki/Block-Structure#partset>
    ///
    /// PartSetHeader in protobuf is defined as never nil using the gogoproto
    /// annotations. This does not translate to Rust, but we can indicate this
    /// in the domain type.
    pub part_set_header: PartSetHeader,
}

impl Protobuf<RawBlockId> for Id {}

impl TryFrom<RawBlockId> for Id {
    type Error = Error;

    fn try_from(value: RawBlockId) -> Result<Self, Self::Error> {
        if value.part_set_header.is_none() {
            return Err(Error::invalid_part_set_header(
                "part_set_header is None".to_string(),
            ));
        }
        Ok(Self {
            hash: value.hash.try_into()?,
            part_set_header: value.part_set_header.unwrap().try_into()?,
        })
    }
}

impl From<Id> for RawBlockId {
    fn from(value: Id) -> Self {
        // https://github.com/tendermint/tendermint/blob/1635d1339c73ae6a82e062cd2dc7191b029efa14/types/block.go#L1204
        // The Go implementation encodes a nil value into an empty struct. We try our best to
        // anticipate an empty struct by using the default implementation which would otherwise be
        // invalid.
        if value == Id::default() {
            RawBlockId {
                hash: vec![],
                part_set_header: Some(RawPartSetHeader {
                    total: 0,
                    hash: vec![],
                }),
            }
        } else {
            RawBlockId {
                hash: value.hash.into(),
                part_set_header: Some(value.part_set_header.into()),
            }
        }
    }
}

impl TryFrom<RawCanonicalBlockId> for Id {
    type Error = Error;

    fn try_from(value: RawCanonicalBlockId) -> Result<Self, Self::Error> {
        if value.part_set_header.is_none() {
            return Err(Error::invalid_part_set_header(
                "part_set_header is None".to_string(),
            ));
        }
        Ok(Self {
            hash: value.hash.try_into()?,
            part_set_header: value.part_set_header.unwrap().try_into()?,
        })
    }
}

impl From<Id> for RawCanonicalBlockId {
    fn from(value: Id) -> Self {
        RawCanonicalBlockId {
            hash: value.hash.as_bytes().to_vec(),
            part_set_header: Some(value.part_set_header.into()),
        }
    }
}

impl Id {
    /// Get a shortened 12-character prefix of a block ID (ala git)
    pub fn prefix(&self) -> String {
        let mut result = self.to_string();
        result.truncate(PREFIX_LENGTH);
        result
    }
}

// TODO: match gaia serialization? e.g `D2F5991B98D708FD2C25AA2BEBED9358F24177DE:1:C37A55FB95E9`
impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.hash)
    }
}

// TODO: match gaia serialization?
impl FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(Self {
            hash: Hash::from_hex_upper(Algorithm::Sha256, s)?,
            part_set_header: PartSetHeader::default(),
        })
    }
}

/// Parse `block::Id` from a type
pub trait ParseId {
    /// Parse `block::Id`, or return an `Error` if parsing failed
    fn parse_block_id(&self) -> Result<Id, Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_SHA256_ID: &str =
        "26C0A41F3243C6BCD7AD2DFF8A8D83A71D29D307B5326C227F734A1A512FE47D";

    #[test]
    fn parses_hex_strings() {
        let id = Id::from_str(EXAMPLE_SHA256_ID).unwrap();
        assert_eq!(
            id.hash.as_bytes(),
            b"\x26\xC0\xA4\x1F\x32\x43\xC6\xBC\xD7\xAD\x2D\xFF\x8A\x8D\x83\xA7\
              \x1D\x29\xD3\x07\xB5\x32\x6C\x22\x7F\x73\x4A\x1A\x51\x2F\xE4\x7D"
                .as_ref()
        );
    }

    #[test]
    fn serializes_hex_strings() {
        let id = Id::from_str(EXAMPLE_SHA256_ID).unwrap();
        assert_eq!(&id.to_string(), EXAMPLE_SHA256_ID)
    }
}
