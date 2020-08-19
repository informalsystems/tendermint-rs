//! Tendermint node IDs

use crate::{
    error::{Error, Kind},
    public_key::Ed25519,
};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::{
    fmt::{self, Debug, Display},
    str::FromStr,
};
use subtle::{self, ConstantTimeEq};
use subtle_encoding::hex;

/// Length of a Node ID in bytes
pub const LENGTH: usize = 20;

/// Node IDs
#[allow(clippy::derive_hash_xor_eq)]
#[derive(Copy, Clone, Eq, Hash, PartialOrd, Ord)]
pub struct Id([u8; LENGTH]);

impl Id {
    /// Create a new Node ID from raw bytes
    pub fn new(bytes: [u8; LENGTH]) -> Id {
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

impl Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "node::Id({})", self)
    }
}

impl From<Ed25519> for Id {
    fn from(pk: Ed25519) -> Id {
        let digest = Sha256::digest(pk.as_bytes());
        let mut bytes = [0u8; LENGTH];
        bytes.copy_from_slice(&digest[..LENGTH]);
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
            .map_err(|_| Kind::Parse)?;

        if bytes.len() != LENGTH {
            return Err(Kind::Parse.into());
        }

        let mut result_bytes = [0u8; LENGTH];
        result_bytes.copy_from_slice(&bytes);
        Ok(Id(result_bytes))
    }
}

impl PartialEq for Id {
    fn eq(&self, other: &Id) -> bool {
        self.ct_eq(other).into()
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|_| {
            de::Error::custom(format!(
                "expected {}-character hex string, got {:?}",
                LENGTH * 2,
                s
            ))
        })
    }
}

impl Serialize for Id {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}
