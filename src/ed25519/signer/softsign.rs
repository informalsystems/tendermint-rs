// ed25519-dalek software-based signer
//
// This is mainly intended for testing/CI. Ideally real validators will use HSMs

use signatory::{encoding::Decode, Ed25519Seed, PublicKeyed};
use signatory_dalek::Ed25519Signer;
use subtle_encoding::IDENTITY;

use config::provider::softsign::SoftSignConfig;
use ed25519::{KeyRing, PublicKey, Signer};
use error::Error;

/// Label for ed25519-dalek provider
// TODO: use a non-string type for these, e.g. an enum
pub const DALEK_PROVIDER_LABEL: &str = "dalek";

/// Create software-backed Ed25519 signer objects from the given configuration
pub fn init(keyring: &mut KeyRing, configs: &[SoftSignConfig]) -> Result<(), Error> {
    for config in configs {
        let seed = Ed25519Seed::decode_from_file(config.path.as_path(), IDENTITY).map_err(|e| {
            err!(
                ConfigError,
                "can't open {}: {}",
                config.path.as_path().display(),
                e
            )
        })?;

        let signer = Box::new(Ed25519Signer::from(&seed));
        let public_key = PublicKey::from(signer.public_key()?);

        keyring.add(
            public_key,
            Signer::new(DALEK_PROVIDER_LABEL, config.id.clone(), signer),
        )?;
    }

    Ok(())
}
