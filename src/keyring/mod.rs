//! Signing keyring. Presently specialized for Ed25519.

mod ed25519;
mod providers;

use self::ed25519::Signer;
pub use self::providers::SigningProvider;
use crate::{
    chain,
    config::provider::ProviderConfig,
    error::{KmsError, KmsErrorKind::*},
};
use std::{collections::BTreeMap, sync::RwLock};
use subtle_encoding;
use tendermint::TendermintKey;

#[cfg(feature = "ledgertm")]
use self::ed25519::ledgertm;
#[cfg(feature = "softsign")]
use self::ed25519::softsign;
#[cfg(feature = "yubihsm")]
use self::ed25519::yubihsm;

/// File encoding for software-backed secret keys
pub type SecretKeyEncoding = subtle_encoding::Base64;

lazy_static! {
    static ref GLOBAL_KEYRING: RwLock<KeyRing> = RwLock::new(KeyRing(BTreeMap::default()));
}

pub struct KeyRing(BTreeMap<TendermintKey, Signer>);

impl KeyRing {
    /// Create a keyring from the given provider configuration
    pub fn load_from_config(config: &ProviderConfig) -> Result<(), KmsError> {
        let mut keyring = GLOBAL_KEYRING.write().unwrap();

        // Clear the current global keyring
        if !keyring.0.is_empty() {
            info!("[keyring:*] Clearing keyring");
            keyring.0.clear();
        }

        #[cfg(feature = "softsign")]
        softsign::init(&mut keyring, &config.softsign)?;

        #[cfg(feature = "yubihsm")]
        yubihsm::init(&mut keyring, &config.yubihsm)?;

        #[cfg(feature = "ledgertm")]
        ledgertm::init(&mut keyring, &config.ledgertm)?;

        if keyring.0.is_empty() {
            fail!(ConfigError, "no signing keys configured!")
        } else {
            Ok(())
        }
    }

    /// Add a key to the keyring, returning an error if we already have a
    /// signer registered for the given public key
    pub(super) fn add(
        &mut self,
        public_key: TendermintKey,
        signer: Signer,
    ) -> Result<(), KmsError> {
        let provider = signer.provider();

        // Generate string for displaying the HRP
        let chains = signer
            .chain_ids()
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        // Use the correct HRP for the configured chain if available
        let public_key_serialized = if signer.chain_ids().len() == 1 {
            let chain_id = signer.chain_ids()[0];
            match chain::key::serialize(chain_id, public_key) {
                Some(bech32) => bech32,
                None => fail!(InvalidKey, "unknown chain ID: {}", chain_id),
            }
        } else {
            public_key.to_bech32("")
        };

        info!(
            "[keyring:{}:{}] added validator key {}",
            provider, chains, public_key_serialized
        );

        if let Some(other) = self.0.insert(public_key, signer) {
            fail!(
                InvalidKey,
                "[keyring:{}:{}] duplicate key {} already registered as {}:{:?}",
                provider,
                chains,
                public_key_serialized,
                other.provider(),
                other.chain_ids()
            )
        } else {
            Ok(())
        }
    }

    /// Get the default public key for this keyring
    pub fn default_pubkey() -> Result<TendermintKey, KmsError> {
        let keyring = GLOBAL_KEYRING.read().unwrap();
        let mut keys = keyring.0.keys();

        if keys.len() == 1 {
            Ok(*keys.next().unwrap())
        } else {
            fail!(InvalidKey, "expected only one key in keyring");
        }
    }

    /// Sign a message using the secret key associated with the given public key
    /// (if it is in our keyring)
    pub fn sign_ed25519(
        public_key: Option<&TendermintKey>,
        msg: &[u8],
    ) -> Result<ed25519::Signature, KmsError> {
        let keyring = GLOBAL_KEYRING.read().unwrap();
        let signer: &Signer = match public_key {
            Some(public_key) => keyring
                .0
                .get(public_key)
                .ok_or_else(|| err!(InvalidKey, "not in keyring: {}", public_key.to_bech32("")))?,
            None => {
                let mut vals = keyring.0.values();

                if vals.len() > 1 {
                    fail!(SigningError, "expected only one key in keyring");
                } else {
                    vals.next()
                        .ok_or_else(|| err!(InvalidKey, "could not get only signer"))?
                }
            }
        };

        signer.sign(msg)
    }
}
