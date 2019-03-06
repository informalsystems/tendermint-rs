//! YubiHSM2-based signer

use crate::{
    config::provider::yubihsm::YubihsmConfig,
    error::{KmsError, KmsErrorKind::*},
    keyring::{ed25519::Signer, KeyRing},
};
use signatory::PublicKeyed;

/// Label for YubiHSM provider
// TODO: use a non-string type for these, e.g. an enum
pub const YUBIHSM_PROVIDER_LABEL: &str = "yubihsm";

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

        keyring.add(
            signer.public_key()?,
            Signer::new(YUBIHSM_PROVIDER_LABEL, config.id.clone(), Box::new(signer)),
        )?;
    }

    Ok(())
}
