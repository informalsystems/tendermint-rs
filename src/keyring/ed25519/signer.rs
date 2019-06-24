//! Wrapper for Ed25519 signers

use crate::{
    error::{Error, ErrorKind::*},
    keyring::SigningProvider,
};
use signatory::ed25519::Signature;
use std::sync::Arc;
use tendermint::TendermintKey;

/// Trait object wrapper for an Ed25519 signers
#[derive(Clone)]
pub struct Signer {
    /// Provider for this signer
    provider: SigningProvider,

    /// Tendermint public key
    public_key: TendermintKey,

    /// Signer trait object
    signer: Arc<Box<dyn signatory::Signer<Signature> + Send + Sync>>,
}

impl Signer {
    /// Create a new signer
    pub fn new(
        provider: SigningProvider,
        public_key: TendermintKey,
        signer: Box<dyn signatory::Signer<Signature> + Send + Sync>,
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
    pub fn sign(&self, msg: &[u8]) -> Result<Signature, Error> {
        Ok(self
            .signer
            .try_sign(msg)
            .map_err(|e| err!(SigningError, "{}", e))?)
    }
}
