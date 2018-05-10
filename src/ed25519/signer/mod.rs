use signatory::ed25519::{Signature, Signer as SignerTrait};

#[cfg(feature = "dalek-provider")]
pub mod dalek;

#[cfg(feature = "yubihsm-provider")]
pub mod yubihsm;

use error::Error;
use super::PublicKey;

/// Wrapper for an Ed25519 signing provider (i.e. trait object)
pub struct Signer {
    /// Name of the signature provider for this key
    pub provider_name: &'static str,

    /// ID which identifies this key (should be unique-per-provider)
    pub key_id: String,

    /// Signer trait object
    provider: Box<SignerTrait>,
}

impl Signer {
    /// Create a new signer
    pub fn new(provider_name: &'static str, key_id: String, provider: Box<SignerTrait>) -> Self {
        Self {
            provider_name,
            key_id,
            provider,
        }
    }

    /// Obtain the Ed25519 public key which corresponds to this signer's private key
    pub fn public_key(&self) -> Result<PublicKey, Error> {
        Ok(self.provider
            .public_key()
            .map_err(|e| err!(InvalidKey, "{}", e))?
            .into())
    }

    /// Sign the given message using this signer
    #[inline]
    pub fn sign(&self, msg: &[u8]) -> Result<Signature, Error> {
        Ok(self.provider
            .sign(msg)
            .map_err(|e| err!(SigningError, "{}", e))?)
    }
}
