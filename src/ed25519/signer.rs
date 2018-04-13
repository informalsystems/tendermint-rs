use signatory::ed25519::Signer as SignerTrait;
#[cfg(feature = "dalek-provider")]
use signatory::ed25519::FromSeed;
#[cfg(feature = "dalek-provider")]
use signatory::providers::dalek::Ed25519Signer as DalekSigner;

use error::Error;
use super::PublicKey;

/// Wrapper for an Ed25519 signing provider (i.e. trait object)
pub struct Signer(Box<SignerTrait>);

impl Signer {
    /// Create a new signer with a software-based backend (i.e. ed25519-dalek)
    #[cfg(feature = "dalek-provider")]
    pub fn new_with_soft_backend(seed: &[u8]) -> Result<Self, Error> {
        Ok(Signer(Box::new(DalekSigner::from_seed(seed).unwrap())))
    }

    /// Obtain the Ed25519 public key which corresponds to this signer's private key
    pub fn public_key(&mut self) -> Result<PublicKey, Error> {
        Ok(self.0
            .public_key()
            .map_err(|e| err!(InvalidKey, "{}", e))?
            .into())
    }
}
