use crate::{
    chain,
    error::{KmsError, KmsErrorKind::*},
    keyring::SigningProvider,
};
use signatory::{self, ed25519::Signature, Signer as SignerTrait};

/// Wrapper for an Ed25519 signing provider (i.e. trait object)
pub struct Signer {
    /// Provider for this signer
    provider: SigningProvider,

    /// Chains this key is authorized to be used from
    chain_ids: Vec<chain::Id>,

    /// Signer trait object
    signer: Box<dyn SignerTrait<Signature>>,
}

impl Signer {
    /// Create a new signer
    pub fn new(
        provider: SigningProvider,
        chain_ids: &[chain::Id],
        signer: Box<dyn SignerTrait<Signature>>,
    ) -> Self {
        Self {
            provider,
            chain_ids: chain_ids.to_vec(),
            signer,
        }
    }

    /// Get the provider for this signer
    pub fn provider(&self) -> SigningProvider {
        self.provider
    }

    /// Get the chains this signer is authorized to be used on
    pub fn chain_ids(&self) -> &[chain::Id] {
        &self.chain_ids
    }

    /// Sign the given message using this signer
    pub fn sign(&self, msg: &[u8]) -> Result<Signature, KmsError> {
        Ok(signatory::sign(self.signer.as_ref(), msg).map_err(|e| err!(SigningError, "{}", e))?)
    }
}
