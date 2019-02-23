//! YubiHSM2-based signer

use crate::{
    config::provider::yubihsm::YubihsmConfig,
    error::{KmsError, KmsErrorKind::*},
    keyring::{ed25519::Signer, KeyRing},
};
use signatory::PublicKeyed;

/// Label for ed25519-dalek provider
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

    let yubihsm_config = &yubihsm_configs[0];
    let connector = yubihsm::UsbConnector::create(&yubihsm_config.usb_config())?;
    let session =
        yubihsm::signatory::Session::create(connector, yubihsm_config.auth.credentials())?;

    for key_config in &yubihsm_config.keys {
        let provider = Box::new(session.ed25519_signer(key_config.key)?);

        keyring.add(
            provider.public_key()?,
            Signer::new(YUBIHSM_PROVIDER_LABEL, key_config.id.clone(), provider),
        )?;
    }

    Ok(())
}
