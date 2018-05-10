use signatory::ed25519::Signer as SignerTrait;

#[cfg(feature = "dalek-provider")]
pub mod dalek;

// TODO: #[cfg(feature = "yubihsm-provider")] pub mod yubihsm;

use super::PublicKey;
use error::Error;

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
    pub fn public_key(&mut self) -> Result<PublicKey, Error> {
        Ok(self.provider
            .public_key()
            .map_err(|e| err!(InvalidKey, "{}", e))?
            .into())
    }
}
