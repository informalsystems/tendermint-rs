//! YubiHSM2-based signer

use signatory::PublicKeyed;
use signatory_yubihsm::{self, KeyId};

use crate::{
    config::provider::yubihsm::YubihsmConfig,
    error::{KmsError, KmsErrorKind::*},
    keyring::{ed25519::Signer, KeyRing},
};

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
    let connector = signatory_yubihsm::yubihsm::UsbConnector::new(&yubihsm_config.usb_config())?;
    let session = signatory_yubihsm::Session::create(connector, yubihsm_config.auth.credentials())?;

    for key_config in &yubihsm_config.keys {
        let provider = Box::new(session.ed25519_signer(KeyId(key_config.key))?);

        keyring.add(
            provider.public_key()?,
            Signer::new(YUBIHSM_PROVIDER_LABEL, key_config.id.clone(), provider),
        )?;
    }

    Ok(())
}
