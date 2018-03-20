use failure::Fail;
use signatory::ed25519::Signer as SignerTrait;
#[cfg(feature = "dalek-provider")]
use signatory::providers::DalekSigner;

use error::{Error, ErrorKind};
use super::PublicKey;

pub struct Signer(Box<SignerTrait>);

impl Signer {
    #[cfg(feature = "dalek-provider")]
    pub fn dalek(seed: &[u8]) -> Result<Self, Error> {
        Ok(Signer(Box::new(DalekSigner::from_seed(seed).unwrap())))
    }

    pub fn public_key(&mut self) -> Result<PublicKey, Error> {
        Ok(self.0
            .public_key()
            .map_err(|e| e.kind().context(ErrorKind::SignerError))?
            .into())
    }
}
