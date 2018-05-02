use signatory::ed25519::Signature;
use std::collections::HashMap;

use config::ProviderConfig;
use error::Error;
use std::panic::RefUnwindSafe;
use super::{PublicKey, Signer};

#[cfg(feature = "dalek-provider")]
use super::signer::dalek;

#[cfg(feature = "yubihsm-provider")]
use super::signer::yubihsm;

pub struct Keyring {
    keys: HashMap<PublicKey, Signer>,
}

impl Keyring {
    /// Create a keyring from the given provider configuration
    pub fn from_config(config: ProviderConfig) -> Result<Self, Error> {
        let mut signers = vec![];

        #[cfg(feature = "dalek-provider")]
        dalek::create_signers(&mut signers, config.dalek)?;

        #[cfg(feature = "yubihsm-provider")]
        yubihsm::create_signers(&mut signers, &config.yubihsm)?;

        Self::from_signers(signers)
    }

    /// Create a keyring from the given vector of signer objects
    pub fn from_signers(signers: Vec<Signer>) -> Result<Self, Error> {
        let mut keys = HashMap::new();

        for mut signer in signers {
            let public_key = signer.public_key()?;
            debug!(
                "Added {}:{} {}",
                signer.provider_name, signer.key_id, &public_key
            );
            keys.insert(public_key, signer);
        }

        Ok(Self { keys })
    }

    /// Sign a message using the secret key associated with the given public key
    /// (if it is in our keyring)
    pub fn sign(&self, public_key: &PublicKey, msg: &[u8]) -> Result<Signature, Error> {
        let signer = self.keys
            .get(public_key)
            .ok_or_else(|| err!(InvalidKey, "not in keyring: {}", public_key))?;

        signer.sign(msg)
    }
}

// TODO: push this down and enforce it inside of Signatory.
// Right now it just "happens to be true"
impl RefUnwindSafe for Keyring {}
