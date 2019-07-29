use super::parts;
use crate::{
    error::Error,
    hash::{Algorithm, Hash},
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::{self, FromStr},
};

/// Length of a block ID prefix displayed for debugging purposes
pub const PREFIX_LENGTH: usize = 10;

/// Block identifiers which contain two distinct Merkle roots of the block,
/// as well as the number of parts in the block.
///
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#blockid>
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
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
    pub parts: Option<parts::Header>,
}

impl Id {
    /// Create a new `Id` from a hash byte slice
    pub fn new(hash: Hash, parts: Option<parts::Header>) -> Self {
        Self { hash, parts }
    }

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
        Ok(Self::new(Hash::from_hex_upper(Algorithm::Sha256, s)?, None))
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
            id.hash.as_bytes().unwrap(),
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
