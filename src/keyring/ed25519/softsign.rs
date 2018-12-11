/// ed25519-dalek software-based signer
///
/// This is mainly intended for testing/CI. Ideally real validators will use HSMs
use signatory::{ed25519, encoding::Decode, PublicKeyed};
use signatory_dalek::Ed25519Signer;
use subtle_encoding::IDENTITY;

use super::Signer;
use crate::{
    config::provider::softsign::SoftSignConfig,
    error::{KmsError, KmsErrorKind::*},
    keyring::KeyRing,
};

/// Label for ed25519-dalek provider
// TODO: use a non-string type for these, e.g. an enum
pub const DALEK_PROVIDER_LABEL: &str = "dalek";

/// Create software-backed Ed25519 signer objects from the given configuration
pub fn init(keyring: &mut KeyRing, configs: &[SoftSignConfig]) -> Result<(), KmsError> {
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

        let provider = Box::new(Ed25519Signer::from(&seed));

        keyring.add(
            provider.public_key()?,
            Signer::new(DALEK_PROVIDER_LABEL, config.id.clone(), provider),
        )?;
    }

    Ok(())
}
