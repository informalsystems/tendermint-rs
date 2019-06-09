//! YubiHSM2 signing provider

use crate::{
    chain,
    config::provider::yubihsm::YubihsmConfig,
    error::{Error, ErrorKind::*},
    keyring::{ed25519::Signer, SigningProvider},
};
use signatory::PublicKeyed;
use tendermint::TendermintKey;

/// Create hardware-backed YubiHSM signer objects from the given configuration
pub fn init(
    chain_registry: &mut chain::Registry,
    yubihsm_configs: &[YubihsmConfig],
) -> Result<(), Error> {
    if yubihsm_configs.is_empty() {
        return Ok(());
    }

    // TODO(tarcieri): support for multiple YubiHSMs per host?
    if yubihsm_configs.len() != 1 {
        fail!(
            ConfigError,
            "expected one [yubihsm.provider] in config, found: {}",
            yubihsm_configs.len()
        );
    }

    for config in &yubihsm_configs[0].keys {
        let signer = yubihsm::ed25519::Signer::create(crate::yubihsm::client().clone(), config.key)
            .map_err(|_| Error::from(InvalidKey))?;

        // TODO(tarcieri): support for adding account keys into keyrings
        let public_key = TendermintKey::ConsensusKey(
            signer
                .public_key()
                .map_err(|_| Error::from(InvalidKey))?
                .into(),
        );

        let signer = Signer::new(SigningProvider::Yubihsm, public_key, Box::new(signer));

        for chain_id in &config.chain_ids {
            chain_registry.add_to_keyring(chain_id, signer.clone())?;
        }
    }

    Ok(())
}
