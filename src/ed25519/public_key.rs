use signatory::ed25519::PublicKey as SignatoryKey;
pub use signatory::ed25519::PUBLIC_KEY_SIZE;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct PublicKey(SignatoryKey);

impl PublicKey {
    /// Obtain public key as a byte array reference
    #[inline]
    pub fn as_bytes(&self) -> &[u8; PUBLIC_KEY_SIZE] {
        self.0.as_bytes()
    }

    /// Convert public key into owned byte array
    #[inline]
    pub fn into_bytes(self) -> [u8; PUBLIC_KEY_SIZE] {
        self.0.into_bytes()
    }
}

// TODO: public key serialization formats (cosmos-bech32)
impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ed25519(")?;

        for (i, byte) in self.as_bytes().iter().enumerate() {
            write!(f, "{:02x}", byte)?;
            write!(f, "{}", if i == PUBLIC_KEY_SIZE - 1 { ")" } else { ":" })?;
        }

        Ok(())
    }
}

impl From<SignatoryKey> for PublicKey {
    fn from(key: SignatoryKey) -> PublicKey {
        PublicKey(key)
    }
}
