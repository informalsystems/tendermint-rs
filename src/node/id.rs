//! Tendermint node IDs

use crate::error::Error;
#[cfg(feature = "serde")]
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use signatory::ed25519;
use std::{
    fmt::{self, Display},
    str::FromStr,
};
use subtle::{self, ConstantTimeEq};
use subtle_encoding::hex;

/// Size of a Node ID in bytes
pub const ID_LENGTH: usize = 20;

/// Node IDs
#[derive(Copy, Clone, Debug, Hash)]
pub struct Id([u8; ID_LENGTH]);

impl Id {
    /// Create a new Node ID from raw bytes
    pub fn new(bytes: [u8; ID_LENGTH]) -> Id {
        Id(bytes)
    }

    /// Borrow the node ID as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

impl AsRef<[u8]> for Id {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ConstantTimeEq for Id {
    #[inline]
    fn ct_eq(&self, other: &Id) -> subtle::Choice {
        self.as_bytes().ct_eq(other.as_bytes())
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02X}", byte)?;
        }
        Ok(())
    }
}

impl From<ed25519::PublicKey> for Id {
    fn from(pk: ed25519::PublicKey) -> Id {
        let digest = Sha256::digest(pk.as_bytes());
        let mut bytes = [0u8; ID_LENGTH];
        bytes.copy_from_slice(&digest[..ID_LENGTH]);
        Id(bytes)
    }
}

/// Decode Node ID from hex
impl FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Accept either upper or lower case hex
        let bytes = hex::decode_upper(s)
            .or_else(|_| hex::decode(s))
            .map_err(|_| Error::Parse)?;

        if bytes.len() != ID_LENGTH {
            return Err(Error::Parse);
        }

        let mut result_bytes = [0u8; ID_LENGTH];
        result_bytes.copy_from_slice(&bytes);
        Ok(Id(result_bytes))
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|_| {
            de::Error::custom(format!(
                "expected {}-character hex string, got {:?}",
                ID_LENGTH * 2,
                s
            ))
        })
    }
}

#[cfg(feature = "serde")]
impl Serialize for Id {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}
