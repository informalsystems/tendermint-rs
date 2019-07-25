//! ed25519-dalek software-based signer
//!
//! This is mainly intended for testing/CI. Ideally real validators will use HSMs

use super::Signer;
use crate::{
    chain,
    config::provider::softsign::{KeyFormat, SoftsignConfig},
    error::{Error, ErrorKind::*},
    keyring::{SecretKeyEncoding, SigningProvider},
};
use signatory::{ed25519, encoding::Decode, PublicKeyed};
use signatory_dalek::Ed25519Signer;
use std::{fs, process};
use tendermint::{config::PrivValidatorKey, PrivateKey, TendermintKey};

/// Create software-backed Ed25519 signer objects from the given configuration
pub fn init(chain_registry: &mut chain::Registry, configs: &[SoftsignConfig]) -> Result<(), Error> {
    if configs.is_empty() {
        return Ok(());
    }

    // TODO(tarcieri): support for multiple softsign keys?
    if configs.len() != 1 {
        fail!(
            ConfigError,
            "expected one [softsign.provider] in config, found: {}",
            configs.len()
        );
    }

    let config = &configs[0];
    let key_format = config.key_format.as_ref().cloned().unwrap_or_default();

    let seed = match key_format {
        KeyFormat::Base64 => {
            let base64 = fs::read_to_string(&config.path).map_err(|e| {
                err!(
                    ConfigError,
                    "couldn't read key from {}: {}",
                    &config.path.as_ref().display(),
                    e
                )
            })?;

            // TODO(tarcieri): constant-time string trimming
            let base64_trimmed = base64.trim_end();

            ed25519::Seed::decode_from_str(base64_trimmed, &SecretKeyEncoding::default()).map_err(
                |e| {
                    err!(
                        ConfigError,
                        "can't decode key from {}: {}",
                        config.path.as_ref().display(),
                        e
                    )
                },
            )?
        }
        KeyFormat::Raw => {
            let bytes = fs::read(&config.path).map_err(|e| {
                err!(
                    ConfigError,
                    "couldn't read key from {}: {}",
                    &config.path.as_ref().display(),
                    e
                )
            })?;

            ed25519::Seed::from_bytes(&bytes).ok_or_else(|| {
                err!(
                    ConfigError,
                    "malformed 'raw' softsign key: {}",
                    config.path.as_ref().display(),
                )
            })?
        }
        KeyFormat::Json => {
            let private_key = PrivValidatorKey::load_json_file(&config.path)
                .unwrap_or_else(|e| {
                    status_err!("couldn't load {}: {}", config.path.as_ref().display(), e);
                    process::exit(1);
                })
                .priv_key;

            match private_key {
                PrivateKey::Ed25519(pk) => pk.to_seed(),
            }
        }
    };

    let provider = Ed25519Signer::from(&seed);

    // TODO(tarcieri): support for adding account keys into keyrings
    let public_key = TendermintKey::ConsensusKey(
        provider
            .public_key()
            .map_err(|_| Error::from(InvalidKey))?
            .into(),
    );

    let signer = Signer::new(SigningProvider::SoftSign, public_key, Box::new(provider));

    for chain_id in &config.chain_ids {
        chain_registry.add_to_keyring(chain_id, signer.clone())?;
    }

    Ok(())
}
