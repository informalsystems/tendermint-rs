//! Tendermint blockchain identifiers

use alloc::borrow::Cow;
use core::{
    cmp::Ordering,
    convert::TryFrom,
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    str::{self, FromStr},
};

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use tendermint_proto::Protobuf;

use crate::{error::Error, prelude::*};

/// Maximum length of a `chain::Id` name. Matches `MaxChainIDLen` from:
/// <https://github.com/tendermint/tendermint/blob/develop/types/genesis.go>
// TODO: update this when `chain::Id` is derived from a digest output
pub const MAX_LENGTH: usize = 50;

/// Chain identifier (e.g. 'gaia-9000')
#[derive(Clone)]
pub struct Id(Cow<'static, str>);

impl Protobuf<String> for Id {}

impl TryFrom<String> for Id {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() || value.len() > MAX_LENGTH {
            return Err(Error::length());
        }

        validate(&value).map_err(|_| Error::parse("chain id charset".to_string()))?;
        Ok(Id(Cow::Owned(value)))
    }
}

impl From<Id> for String {
    fn from(value: Id) -> Self {
        value.0.into_owned()
    }
}

impl Id {
    /// Create a new chain ID constant.
    ///
    /// This implementation is `const`-friendly and can be used to define
    /// chain ID constants. Use the `FromStr` implementation if you'd like
    /// a heap-allocated chain ID instead.
    ///
    /// Panics if the chain ID is not valid.
    pub const fn new(id: &'static str) -> Self {
        if id.is_empty() || id.len() > MAX_LENGTH {
            panic!("chain ID has invalid length");
        }

        if validate(id).is_err() {
            panic!("chain ID is invalid");
        }

        Id(Cow::Borrowed(id))
    }

    /// Get the chain ID as a `str`
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    /// Get the chain ID as a raw bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "chain::Id({})", self.as_str())
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
        self.as_str().hash(state)
    }
}

impl PartialOrd for Id {
    fn partial_cmp(&self, other: &Id) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Id {
    fn cmp(&self, other: &Id) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialEq for Id {
    fn eq(&self, other: &Id) -> bool {
        self.as_str() == other.as_str()
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

/// Validate that a given input string is a well-formed chain ID.
const fn validate(s: &str) -> Result<(), ()> {
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if !matches!(bytes[i], b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.') {
            return Err(());
        }

        i += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorDetail;

    const EXAMPLE_CHAIN_ID: &str = "gaia-9000";
    const MAX_LENGTH_CHAIN_ID: &str = "01234567890123456789012345678901234567890123456789";
    const OVERLENGTH_CHAIN_ID: &str = "012345678901234567890123456789012345678901234567890";

    #[test]
    fn from_str_parses_valid_chain_ids() {
        assert_eq!(
            EXAMPLE_CHAIN_ID.parse::<Id>().unwrap().as_str(),
            EXAMPLE_CHAIN_ID
        );

        assert_eq!(
            MAX_LENGTH_CHAIN_ID.parse::<Id>().unwrap().as_str(),
            MAX_LENGTH_CHAIN_ID
        );
    }

    #[test]
    fn from_str_rejects_empty_chain_ids() {
        assert!(
            matches!(
                "".parse::<Id>().unwrap_err().detail(),
                ErrorDetail::Length(_)
            ),
            "expected length error"
        );
    }

    #[test]
    fn from_str_rejects_overlength_chain_ids() {
        assert!(
            matches!(
                OVERLENGTH_CHAIN_ID.parse::<Id>().unwrap_err().detail(),
                ErrorDetail::Length(_)
            ),
            "expected length error"
        );
    }

    #[test]
    fn new_parses_valid_chain_ids() {
        assert_eq!(Id::new(EXAMPLE_CHAIN_ID).as_str(), EXAMPLE_CHAIN_ID);
        assert_eq!(Id::new(MAX_LENGTH_CHAIN_ID).as_str(), MAX_LENGTH_CHAIN_ID);
    }

    #[test]
    #[should_panic]
    fn new_rejects_empty_chain_ids() {
        Id::new("");
    }

    #[test]
    #[should_panic]
    fn new_rejects_overlength_chain_ids() {
        Id::new(OVERLENGTH_CHAIN_ID);
    }
}
