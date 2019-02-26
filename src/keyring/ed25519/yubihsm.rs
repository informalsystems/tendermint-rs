//! YubiHSM2-based signer

use crate::{
    config::provider::yubihsm::YubihsmConfig,
    error::{KmsError, KmsErrorKind::*},
    keyring::{ed25519::Signer, KeyRing},
};
use signatory::PublicKeyed;
use yubihsm::signatory::Ed25519Signer;

/// Label for YubiHSM provider
// TODO: use a non-string type for these, e.g. an enum
pub const YUBIHSM_PROVIDER_LABEL: &str = "yubihsm";

/// Create hardware-backed YubiHSM signer objects from the given configuration
pub fn init(keyring: &mut KeyRing, yubihsm_configs: &[YubihsmConfig]) -> Result<(), KmsError> {
    if yubihsm_configs.is_empty() {
        return Ok(());
    }

    if yubihsm_configs.len() != 1 {
        fail!(
            ConfigError,
            "expected one [yubihsm.provider] in config, found: {}",
            yubihsm_configs.len()
        );
    }

    let config = &yubihsm_configs[0];

    let connector = crate::yubihsm::connector();
    let credentials = config.auth.credentials();

    for key_config in &config.keys {
        let hsm = yubihsm::Client::create(connector.clone(), credentials.clone())?;
        let signer = Ed25519Signer::create(hsm, key_config.key)?;

        keyring.add(
            signer.public_key()?,
            Signer::new(
                YUBIHSM_PROVIDER_LABEL,
                key_config.id.clone(),
                Box::new(signer),
            ),
        )?;
    }

    Ok(())
}
