//! ed25519-dalek software-based signer
//!
//! This is mainly intended for testing/CI. Ideally real validators will use HSMs

use super::Signer;
use crate::{
    chain,
    config::provider::softsign::SoftSignConfig,
    error::{Error, ErrorKind::*},
    keyring::SigningProvider,
};
use signatory::{ed25519, encoding::Decode, PublicKeyed};
use signatory_dalek::Ed25519Signer;
use subtle_encoding::IDENTITY;
use tendermint::TendermintKey;

/// Create software-backed Ed25519 signer objects from the given configuration
pub fn init(chain_registry: &mut chain::Registry, configs: &[SoftSignConfig]) -> Result<(), Error> {
    for config in configs {
        let seed =
            ed25519::Seed::decode_from_file(config.path.as_path(), IDENTITY).map_err(|e| {
                err!(
                    ConfigError,
                    "can't open {}: {}",
                    config.path.as_path().display(),
                    e
                )
            })?;

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
    }

    Ok(())
}
