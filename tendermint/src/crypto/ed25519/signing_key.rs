#[cfg(feature = "rust-crypto")]
use super::VerificationKey;

use crate::Error;

#[derive(Clone, Debug)]
pub struct SigningKey([u8; 32]);

impl SigningKey {
    #[allow(dead_code)]
    pub(super) fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    #[cfg(feature = "rust-crypto")]
    pub fn verification_key(&self) -> VerificationKey {
        let privkey = ed25519_consensus::SigningKey::from(self.0);
        let pubkey = privkey.verification_key();
        let pubkey_bytes = pubkey.to_bytes();
        VerificationKey::new(pubkey_bytes)
    }
}

impl TryFrom<&'_ [u8]> for SigningKey {
    type Error = Error;

    fn try_from(slice: &'_ [u8]) -> Result<Self, Self::Error> {
        if slice.len() != 32 {
            return Err(Error::invalid_key("invalid ed25519 key length".into()));
        }
        let mut bytes = [0u8; 32];
        bytes[..].copy_from_slice(slice);
        Ok(Self(bytes))
    }
}

#[cfg(feature = "rust-crypto")]
impl TryFrom<SigningKey> for ed25519_consensus::SigningKey {
    type Error = Error;

    fn try_from(src: SigningKey) -> Result<Self, Error> {
        Ok(ed25519_consensus::SigningKey::from(src.0))
    }
}

#[cfg(feature = "rust-crypto")]
impl From<ed25519_consensus::SigningKey> for SigningKey {
    fn from(sk: ed25519_consensus::SigningKey) -> Self {
        Self::new(sk.to_bytes())
    }
}
