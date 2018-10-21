use signatory::{self, Ed25519Signature, Signer as SignerTrait};

#[cfg(feature = "softsign")]
pub mod softsign;
#[cfg(feature = "yubihsm")]
pub mod yubihsm;

use error::{KmsError, KmsErrorKind::*};

/// Wrapper for an Ed25519 signing provider (i.e. trait object)
pub struct Signer {
    /// Name of the signature provider for this key
    pub provider_name: &'static str,

    /// ID which identifies this key (should be unique-per-provider)
    pub key_id: String,

    /// Signer trait object
    provider: Box<SignerTrait<Ed25519Signature>>,
}

impl Signer {
    /// Create a new signer
    pub fn new(
        provider_name: &'static str,
        key_id: String,
        provider: Box<SignerTrait<Ed25519Signature>>,
    ) -> Self {
        Self {
            provider_name,
            key_id,
            provider,
        }
    }

    /// Sign the given message using this signer
    #[inline]
    pub fn sign(&self, msg: &[u8]) -> Result<Ed25519Signature, KmsError> {
        Ok(
            signatory::sign(self.provider.as_ref(), msg)
                .map_err(|e| err!(SigningError, "{}", e))?,
        )
    }
}
