//! Signing keyring. Presently specialized for Ed25519.

pub mod ed25519;
mod format;
mod providers;

use self::ed25519::Signer;
pub use self::{format::Format, providers::SigningProvider};
use crate::{
    chain,
    config::provider::ProviderConfig,
    error::{Error, ErrorKind::*},
};
use std::collections::BTreeMap;
use subtle_encoding;
use tendermint::TendermintKey;

/// File encoding for software-backed secret keys
pub type SecretKeyEncoding = subtle_encoding::Base64;

/// Signing keyring
pub struct KeyRing {
    /// Keys in the keyring
    keys: BTreeMap<TendermintKey, Signer>,

    /// Formatting configuration when displaying keys (e.g. bech32)
    format: Format,
}

impl KeyRing {
    /// Create a new keyring
    pub fn new(format: Format) -> Self {
        Self {
            keys: BTreeMap::new(),
            format,
        }
    }

    /// Add a key to the keyring, returning an error if we already have a
    /// signer registered for the given public key
    pub fn add(&mut self, signer: Signer) -> Result<(), Error> {
        let provider = signer.provider();
        let public_key = signer.public_key();
        let public_key_serialized = self.format.serialize(public_key);
        let key_type = match public_key {
            TendermintKey::AccountKey(_) => "account",
            TendermintKey::ConsensusKey(_) => "consensus",
        };

        info!(
            "[keyring:{}] added {} key {}",
            provider, key_type, public_key_serialized
        );

        if let Some(other) = self.keys.insert(public_key, signer) {
            fail!(
                InvalidKey,
                "[keyring:{}] duplicate key {} already registered as {}",
                provider,
                public_key_serialized,
                other.provider(),
            )
        } else {
            Ok(())
        }
    }

    /// Get the default public key for this keyring
    pub fn default_pubkey(&self) -> Result<TendermintKey, Error> {
        let mut keys = self.keys.keys();

        if keys.len() == 1 {
            Ok(*keys.next().unwrap())
        } else {
            fail!(InvalidKey, "expected only one key in keyring");
        }
    }

    /// Sign a message using the secret key associated with the given public key
    /// (if it is in our keyring)
    pub fn sign_ed25519(
        &self,
        public_key: Option<&TendermintKey>,
        msg: &[u8],
    ) -> Result<ed25519::Signature, Error> {
        let signer = match public_key {
            Some(public_key) => self
                .keys
                .get(public_key)
                .ok_or_else(|| err!(InvalidKey, "not in keyring: {}", public_key.to_bech32("")))?,
            None => {
                let mut vals = self.keys.values();

                if vals.len() > 1 {
                    fail!(SigningError, "expected only one key in keyring");
                } else {
                    vals.next()
                        .ok_or_else(|| err!(InvalidKey, "keyring is empty"))?
                }
            }
        };

        signer.sign(msg)
    }
}

/// Initialize the keyring from the configuration file
pub fn load_config(registry: &mut chain::Registry, config: &ProviderConfig) -> Result<(), Error> {
    #[cfg(feature = "softsign")]
    ed25519::softsign::init(registry, &config.softsign)?;

    #[cfg(feature = "yubihsm")]
    ed25519::yubihsm::init(registry, &config.yubihsm)?;

    #[cfg(feature = "ledgertm")]
    ed25519::ledgertm::init(registry, &config.ledgertm)?;

    Ok(())
}
