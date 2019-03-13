use crate::error::Error;
use serde::de::{self, Deserialize, Deserializer};
use sha2::{Digest, Sha256};
use signatory::ed25519;
use std::{
    fmt::{self, Display},
    str::FromStr,
};
use subtle::{self, ConstantTimeEq};
use subtle_encoding::hex;

/// Size of a PeerId in bytes
pub const SIZE: usize = 20;

/// SecretConnection Peer IDs
#[derive(Copy, Clone, Debug, Hash)]
pub struct PeerId([u8; SIZE]);

impl PeerId {
    /// Create a new PeerId from raw bytes
    pub fn new(bytes: [u8; SIZE]) -> PeerId {
        PeerId(bytes)
    }

    /// Borrow the Peer ID as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<[u8]> for PeerId {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ConstantTimeEq for PeerId {
    #[inline]
    fn ct_eq(&self, other: &PeerId) -> subtle::Choice {
        self.as_bytes().ct_eq(other.as_bytes())
    }
}

impl Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02X}", byte)?;
        }
        Ok(())
    }
}

impl From<ed25519::PublicKey> for PeerId {
    fn from(pk: ed25519::PublicKey) -> PeerId {
        let digest = Sha256::digest(pk.as_bytes());
        let mut peer_id_bytes = [0u8; SIZE];
        peer_id_bytes.copy_from_slice(&digest[..SIZE]);
        PeerId(peer_id_bytes)
    }
}

/// Decode PeerId from hex
impl FromStr for PeerId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Accept either upper or lower case hex
        let bytes = hex::decode_upper(s)
            .or_else(|_| hex::decode(s))
            .map_err(|_| Error::Parse)?;

        if bytes.len() != SIZE {
            return Err(Error::Parse);
        }

        let mut peer_id_bytes = [0u8; SIZE];
        peer_id_bytes.copy_from_slice(&bytes);
        Ok(PeerId(peer_id_bytes))
    }
}

impl<'de> Deserialize<'de> for PeerId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|_| {
            de::Error::custom(format!(
                "expected {}-character hex string, got {:?}",
                SIZE * 2,
                s
            ))
        })
    }
}
