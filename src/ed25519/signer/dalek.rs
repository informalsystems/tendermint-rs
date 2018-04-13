use signatory::ed25519::FromSeed;
use signatory::providers::dalek::Ed25519Signer as DalekSigner;
use std::fs::File;
use std::io::Read;

use clear_on_drop::ClearOnDrop;
use config::DalekConfig;
use error::Error;
use super::Signer;

/// Label for ed25519-dalek providers
pub const DALEK_PROVIDER_LABEL: &str = "dalek";

/// Create software-backed Ed25519 signers from the given configuration
pub fn create_signers(signers: &mut Vec<Signer>, config: DalekConfig) -> Result<(), Error> {
    for (key_id, key_config) in config.keys {
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

        let signer = Box::new(DalekSigner::from_seed(&key_material).unwrap());
        signers.push(Signer::new(DALEK_PROVIDER_LABEL, key_id, signer));
    }

    Ok(())
}
