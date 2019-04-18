use crate::{
    error::{KmsError, KmsErrorKind::*},
    keyring::SigningProvider,
};
use signatory::{self, ed25519::Signature, Signer as SignerTrait};
use std::sync::Arc;
use tendermint::TendermintKey;

/// Wrapper for an Ed25519 signing provider (i.e. trait object)
#[derive(Clone)]
pub struct Signer {
    /// Provider for this signer
    provider: SigningProvider,

    /// Tendermint public key
    public_key: TendermintKey,

    /// Signer trait object
    signer: Arc<Box<dyn SignerTrait<Signature>>>,
}

impl Signer {
    /// Create a new signer
    pub fn new(
        provider: SigningProvider,
        public_key: TendermintKey,
        signer: Box<dyn SignerTrait<Signature>>,
    ) -> Self {
        Self {
            provider,
            public_key,
            signer: Arc::new(signer),
        }
    }

    /// Get the Tendermint public key for this signer
    pub fn public_key(&self) -> TendermintKey {
        self.public_key
    }

    /// Get the provider for this signer
    pub fn provider(&self) -> SigningProvider {
        self.provider
    }

    /// Sign the given message using this signer
    pub fn sign(&self, msg: &[u8]) -> Result<Signature, KmsError> {
        Ok(self
            .signer
            .sign(msg)
            .map_err(|e| err!(SigningError, "{}", e))?)
    }
}
