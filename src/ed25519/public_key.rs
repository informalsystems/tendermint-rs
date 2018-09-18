use signatory::ed25519::Ed25519PublicKey;
pub use signatory::ed25519::PUBLIC_KEY_SIZE;
use std::fmt::{self, Display};

use error::Error;

/// Ed25519 public keys
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct PublicKey(Ed25519PublicKey);

impl PublicKey {
    /// Convert a bytestring to a public key
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Ok(PublicKey(
            Ed25519PublicKey::from_bytes(bytes).map_err(|e| err!(InvalidKey, "{}", e))?,
        ))
    }

    /// Obtain public key as a byte array reference
    #[inline]
    pub fn as_bytes(&self) -> &[u8; PUBLIC_KEY_SIZE] {
        self.0.as_bytes()
    }
}

// TODO: public key serialization formats (cosmos-bech32)
impl Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ed25519(")?;

        for (i, byte) in self.as_bytes().iter().enumerate() {
            write!(f, "{:02x}", byte)?;
            write!(f, "{}", if i == PUBLIC_KEY_SIZE - 1 { ")" } else { ":" })?;
        }

        Ok(())
    }
}

impl From<Ed25519PublicKey> for PublicKey {
    fn from(key: Ed25519PublicKey) -> PublicKey {
        PublicKey(key)
    }
}
