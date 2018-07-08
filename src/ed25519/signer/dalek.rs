//! ed25519-dalek software-based signer
//!
//! This is mainly intended for testing/CI. Ideally real validators will use HSMs

use signatory::ed25519::{FromSeed,Seed};
use signatory::providers::dalek::Ed25519Signer as DalekSigner;
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

        let signer = Box::new(DalekSigner::from_seed(Seed::from_slice(&key_material).unwrap()));
        signers.push(Signer::new(DALEK_PROVIDER_LABEL, key_id, signer));
    }

    Ok(())
}
