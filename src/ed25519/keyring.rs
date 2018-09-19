use signatory::Ed25519Signature;
use std::{collections::BTreeMap, sync::RwLock, sync::RwLockReadGuard};

use super::{PublicKey, Signer};
use config::ProviderConfig;
use error::Error;

use super::signer::dalek;
#[cfg(feature = "yubihsm-provider")]
use super::signer::yubihsm;

lazy_static! {
    pub static ref GLOBAL_KEYRING: RwLock<KeyRing> = RwLock::new(KeyRing(BTreeMap::default()));
}

pub struct KeyRing(BTreeMap<PublicKey, Signer>);

impl KeyRing {
    /// Create a keyring from the given provider configuration
    pub fn load_from_config(config: &ProviderConfig) -> Result<(), Error> {
        let mut keyring = GLOBAL_KEYRING.write().unwrap();

        // Clear the current global keyring
        if !keyring.0.is_empty() {
            info!("Clearing keyring");
            keyring.0.clear();
        }

        dalek::init(&mut keyring, config.dalek.as_ref())?;

        #[cfg(feature = "yubihsm-provider")]
        yubihsm::init(&mut keyring, &config.yubihsm)?;

        Ok(())
    }

    // TODO(ismail): figure out where we get the pubkey from and delete this:
    pub fn sign_with_only_signer(&self, msg: &[u8]) -> Result<Ed25519Signature, Error> {
        let keyring: RwLockReadGuard<KeyRing>  = GLOBAL_KEYRING.read().unwrap();

        let mut vals = keyring.0.values();
        if vals.len() > 1 {
            return Err(err!(SigningError, "expected only one key in keyring"));
        }
        let signer = vals.next().unwrap();

        signer.sign(msg)
    }

    /// Sign a message using the secret key associated with the given public key
    /// (if it is in our keyring)
    pub fn sign(public_key: &PublicKey, msg: &[u8]) -> Result<Ed25519Signature, Error> {
        let keyring = GLOBAL_KEYRING.read().unwrap();

        let signer = keyring
            .0
            .get(public_key)
            .ok_or_else(|| err!(InvalidKey, "not in keyring: {}", public_key))?;

        signer.sign(msg)
    }

    /// Add a key to the keyring, returning an error if we already have a
    /// signer registered for the given public key
    pub(super) fn add(&mut self, public_key: PublicKey, signer: Signer) -> Result<(), Error> {
        info!(
            "Adding {}:{} {}",
            signer.provider_name, signer.key_id, public_key
        );

        if let Some(other) = self.0.insert(public_key, signer) {
            Err(err!(
                InvalidKey,
                "duplicate signer for {}: {}:{}",
                public_key,
                other.provider_name,
                other.key_id
            ))
        } else {
            Ok(())
        }
    }
}
