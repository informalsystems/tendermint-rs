//! YubiHSM2-based signer

use crate::{
    config::provider::yubihsm::YubihsmConfig,
    error::{KmsError, KmsErrorKind::*},
    keyring::{ed25519::Signer, KeyRing, SigningProvider},
};
use signatory::PublicKeyed;
use tendermint::TendermintKey;

/// Create hardware-backed YubiHSM signer objects from the given configuration
pub fn init(keyring: &mut KeyRing, yubihsm_configs: &[YubihsmConfig]) -> Result<(), KmsError> {
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
        let signer =
            yubihsm::ed25519::Signer::create(crate::yubihsm::client().clone(), config.key)?;

        // TODO(tarcieri): support for adding account keys into keyrings
        let public_key = TendermintKey::ConsensusKey(signer.public_key()?.into());

        keyring.add(
            public_key,
            Signer::new(
                SigningProvider::Yubihsm,
                &config.chain_ids,
                Box::new(signer),
            ),
        )?;
    }

    Ok(())
}
