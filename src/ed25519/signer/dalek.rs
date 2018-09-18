// ed25519-dalek software-based signer
//
// This is mainly intended for testing/CI. Ideally real validators will use HSMs

use signatory::{
    encoding::{Decode, Encoding},
    Ed25519Seed, PublicKeyed,
};
use signatory_dalek::Ed25519Signer;

use config::DalekConfig;
use ed25519::{PublicKey, Signer};
use error::Error;

/// Label for ed25519-dalek provider
pub const DALEK_PROVIDER_LABEL: &str = "dalek";

/// Create software-backed Ed25519 signer objects from the given configuration
// TODO: return an iterator rather than taking an `&mut Vec<Signer>`?
pub fn create_signers(
    signers: &mut Vec<Signer>,
    config_option: Option<&DalekConfig>,
) -> Result<(), Error> {
    if let Some(ref config) = config_option {
        for (key_id, key_config) in &config.keys {
            let seed =
                Ed25519Seed::decode_from_file(&key_config.path, Encoding::Raw).map_err(|e| {
                    err!(
                        ConfigError,
                        "can't open {}: {}",
                        key_config.path.display(),
                        e
                    )
                })?;

            let signer = Box::new(Ed25519Signer::from(&seed));
            let public_key = PublicKey::from_bytes(signer.public_key().unwrap().as_ref()).unwrap();

            signers.push(Signer::new(
                DALEK_PROVIDER_LABEL,
                key_id.to_owned(),
                public_key,
                signer,
            ));
        }
    }

    Ok(())
}
