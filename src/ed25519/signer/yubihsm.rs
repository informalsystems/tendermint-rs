//! YubiHSM2-based signer

use signatory::PublicKeyed;
use signatory_yubihsm::{self, KeyId};

use config::provider::yubihsm::YubihsmConfig;
use ed25519::{KeyRing, PublicKey, Signer};
use error::Error;

/// Label for ed25519-dalek provider
// TODO: use a non-string type for these, e.g. an enum
pub const YUBIHSM_PROVIDER_LABEL: &str = "yubihsm";

/// Create hardware-backed YubiHSM signer objects from the given configuration
pub fn init(keyring: &mut KeyRing, yubihsm_configs: &[YubihsmConfig]) -> Result<(), Error> {
    if yubihsm_configs.is_empty() {
        return Ok(());
    }

    if yubihsm_configs.len() != 1 {
        return Err(err!(
            ConfigError,
            "expected one [yubihsm.provider] in config, found: {}",
            yubihsm_configs.len()
        ));
    }

    let yubihsm_config = &yubihsm_configs[0];
    let connector = signatory_yubihsm::yubihsm::UsbConnector::new(&yubihsm_config.usb_config())?;
    let session = signatory_yubihsm::Session::create(connector, yubihsm_config.auth.credentials())?;

    for key_config in &yubihsm_config.keys {
        let signer = Box::new(session.ed25519_signer(KeyId(key_config.key))?);
        let public_key = PublicKey::from(signer.public_key()?);

        keyring.add(
            public_key,
            Signer::new(YUBIHSM_PROVIDER_LABEL, key_config.id.clone(), signer),
        )?;
    }

    Ok(())
}
