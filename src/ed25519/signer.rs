use failure::Fail;
use signatory::ed25519::Signer as SignerTrait;
use std::sync::Arc;

use error::{Error, ErrorKind};
use super::PublicKey;

pub struct Signer<'a> {
    provider: Arc<SignerTrait + 'a>,
    description: String,
    public_key: PublicKey,
}

impl<'a> Signer<'a> {
    pub fn new<S: SignerTrait + 'a>(mut signer: S, description: String) -> Result<Self, Error> {
        let public_key: PublicKey = signer
            .public_key()
            .map_err(|e| e.kind().context(ErrorKind::SignerError))?
            .into();

        Ok(Self {
            provider: Arc::new(signer),
            description,
            public_key,
        })
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}
