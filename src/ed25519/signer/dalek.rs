//! ed25519-dalek software-based signer
//!
//! This is mainly intended for testing/CI. Ideally real validators will use HSMs

use signatory::ed25519::{self, FromSeed};
use signatory::providers::dalek;
use std::fs::File;
use std::io::Read;

use super::Signer;
use clear_on_drop::ClearOnDrop;
use config::DalekConfig;
use error::Error;

/// Label for ed25519-dalek provider
pub const DALEK_PROVIDER_LABEL: &str = "dalek";

/// Create software-backed Ed25519 signer objects from the given configuration
pub fn create_signers(signers: &mut Vec<Signer>, config: Option<DalekConfig>) -> Result<(), Error> {
    if config.is_none() {
        return Ok(());
    }

    for (key_id, key_config) in config.unwrap().keys {
        let mut file = File::open(&key_config.path).map_err(|e| {
            err!(
                ConfigError,
                "can't open {}: {}",
                key_config.path.display(),
                e
            )
        })?;

        let mut key_material = ClearOnDrop::new(vec![]);
        file.read_to_end(key_material.as_mut())?;

        let seed = ed25519::Seed::from_slice(&key_material).unwrap();
        let signer = Box::new(dalek::Ed25519Signer::from_seed(seed));
        signers.push(Signer::new(DALEK_PROVIDER_LABEL, key_id, signer));
    }

    Ok(())
}
