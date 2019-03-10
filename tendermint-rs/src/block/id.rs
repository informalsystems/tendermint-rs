#[cfg(feature = "serializers")]
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{self, Display},
    str::{self, FromStr},
};

use crate::{algorithm::HashAlgorithm, error::Error, hash::Hash};

/// Block identifiers
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Id {
    /// Hash which identifies this block
    pub hash: Hash, // TODO: parts set header?
}

impl Id {
    /// Create a new `Id` from a hash byte slice
    pub fn new(hash: Hash) -> Self {
        Self { hash }
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
        Ok(Self::new(Hash::from_hex_upper(HashAlgorithm::Sha256, s)?))
    }
}

#[cfg(feature = "serializers")]
impl Serialize for Id {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

#[cfg(feature = "serializers")]
impl<'de> Deserialize<'de> for Id {
    fn deserialize<De: Deserializer<'de>>(deserializer: De) -> Result<Self, De::Error> {
        Self::from_str(&String::deserialize(deserializer)?)
            .map_err(|e| De::Error::custom(format!("{}", e)))
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
            id.hash.as_slice(),
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
