#[cfg(feature = "rust-crypto")]
use super::VerificationKey;

use crate::Error;

#[derive(Clone, Debug)]
pub struct SigningKey([u8; 32]);

impl SigningKey {
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
        ed25519_consensus::SigningKey::try_from(src.0)
            .map_err(|_| Error::invalid_key("malformed Ed25519 private key".into()))
    }
}
