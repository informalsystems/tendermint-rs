//! Tendermint blockchain identifiers

use crate::error::Error;
use crate::prelude::*;
use core::convert::TryFrom;
use core::{
    cmp::Ordering,
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    str::{self, FromStr},
};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use tendermint_proto::Protobuf;

/// Maximum length of a `chain::Id` name. Matches `MaxChainIDLen` from:
/// <https://github.com/tendermint/tendermint/blob/develop/types/genesis.go>
// TODO: update this when `chain::Id` is derived from a digest output
pub const MAX_LENGTH: usize = 50;

/// Chain identifier (e.g. 'gaia-9000')
#[derive(Clone)]
pub struct Id(String);

impl Protobuf<String> for Id {}

impl TryFrom<String> for Id {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() || value.len() > MAX_LENGTH {
            return Err(Error::length());
        }

        for byte in value.as_bytes() {
            match byte {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (),
                _ => return Err(Error::parse("chain id charset".to_string())),
            }
        }

        Ok(Id(value))
    }
}

impl From<Id> for String {
    fn from(value: Id) -> Self {
        value.0
    }
}

impl Id {
    /// Get the chain ID as a `str`
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Get the chain ID as a raw bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_str().as_bytes()
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "chain::Id({})", self.0.as_str())
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> TryFrom<&'a str> for Id {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::try_from(s.to_string())
    }
}

impl FromStr for Id {
    type Err = Error;
    /// Parses string to create a new chain ID
    fn from_str(name: &str) -> Result<Self, Error> {
        Self::try_from(name.to_string())
    }
}

impl Hash for Id {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_str().hash(state)
    }
}

impl PartialOrd for Id {
    fn partial_cmp(&self, other: &Id) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Id {
    fn cmp(&self, other: &Id) -> Ordering {
        self.0.as_str().cmp(other.as_str())
    }
}

impl PartialEq for Id {
    fn eq(&self, other: &Id) -> bool {
        self.0.as_str() == other.as_str()
    }
}

impl Eq for Id {}

impl Serialize for Id {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_str(&String::deserialize(deserializer)?)
            .map_err(|e| D::Error::custom(format!("{}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorDetail;

    const EXAMPLE_CHAIN_ID: &str = "gaia-9000";

    #[test]
    fn parses_valid_chain_ids() {
        assert_eq!(
            EXAMPLE_CHAIN_ID.parse::<Id>().unwrap().as_str(),
            EXAMPLE_CHAIN_ID
        );

        let long_id = String::from_utf8(vec![b'x'; MAX_LENGTH]).unwrap();
        assert_eq!(&long_id.parse::<Id>().unwrap().as_str(), &long_id);
    }

    #[test]
    fn rejects_empty_chain_ids() {
        match "".parse::<Id>().unwrap_err().detail() {
            ErrorDetail::Length(_) => {}
            _ => panic!("expected length error"),
        }
    }

    #[test]
    fn rejects_overlength_chain_ids() {
        let overlong_id = String::from_utf8(vec![b'x'; MAX_LENGTH + 1]).unwrap();
        match overlong_id.parse::<Id>().unwrap_err().detail() {
            ErrorDetail::Length(_) => {}
            _ => panic!("expected length error"),
        }
    }
}
