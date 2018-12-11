use crate::error::{KmsError, KmsErrorKind::*};
use signatory::{self, ed25519::Signature, Signer as SignerTrait};

/// Wrapper for an Ed25519 signing provider (i.e. trait object)
pub struct Signer {
    /// Name of the signature provider for this key
    pub provider_name: &'static str,

    /// ID which identifies this key (should be unique-per-provider)
    pub key_id: String,

    /// Signer trait object
    provider: Box<dyn SignerTrait<Signature>>,
}

impl Signer {
    /// Create a new signer
    pub fn new(
        provider_name: &'static str,
        key_id: String,
        provider: Box<dyn SignerTrait<Signature>>,
    ) -> Self {
        Self {
            provider_name,
            key_id,
            provider,
        }
    }

    /// Sign the given message using this signer
    #[inline]
    pub fn sign(&self, msg: &[u8]) -> Result<Signature, KmsError> {
        Ok(
            signatory::sign(self.provider.as_ref(), msg)
                .map_err(|e| err!(SigningError, "{}", e))?,
        )
    }
}
