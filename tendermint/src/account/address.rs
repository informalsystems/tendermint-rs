//! Bech32-encoded account addresses

use super::Id;
use crate::{error, Error};
use std::{convert::TryInto, fmt, str::FromStr};
use subtle_encoding::bech32;

/// Bech32-encoded account addresses.
///
/// The [`Id`] type provides a raw binary serialization.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Address {
    /// Account ID encoded as Bech32
    bech32: String,

    /// Length of the human-readable prefix of the address
    hrp_length: usize,
}

impl Address {
    /// Create a Bech32-encoded [`Address`] with the given human-readable
    /// prefix and public key hash.
    pub fn new(prefix: &str, id: Id) -> Result<Self, Error> {
        // TODO(tarcieri): ensure this is the proper validation for an account prefix
        if prefix.chars().all(|c| matches!(c, 'a'..='z')) {
            Ok(Self {
                bech32: bech32::encode(prefix, &id.0),
                hrp_length: prefix.len(),
            })
        } else {
            Err(error::Kind::Parse.into())
        }
    }

    /// Get the human-readable prefix of this account.
    pub fn prefix(&self) -> &str {
        &self.bech32[..self.hrp_length]
    }

    /// Decode an account ID from Bech32 to an inner byte value.
    pub fn to_bytes(&self) -> [u8; Id::LENGTH] {
        bech32::decode(&self.bech32)
            .ok()
            .and_then(|result| result.1.try_into().ok())
            .expect("malformed Bech32 Address")
    }
}

impl AsRef<str> for Address {
    fn as_ref(&self) -> &str {
        &self.bech32
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Address").field(&self.as_ref()).finish()
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl FromStr for Address {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let (hrp, bytes) = bech32::decode(s)?;

        if bytes.len() == Id::LENGTH {
            Ok(Self {
                bech32: s.to_owned(),
                hrp_length: hrp.len(),
            })
        } else {
            Err(error::Kind::Parse.into())
        }
    }
}

impl From<Address> for Id {
    fn from(id: Address) -> Id {
        Id::from(&id)
    }
}

impl From<&Address> for Id {
    fn from(id: &Address) -> Id {
        Id(id.to_bytes())
    }
}
