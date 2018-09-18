//! YubiHSM2-based signer
// TODO: finish implementing this!

use config::YubihsmConfig;
use ed25519::KeyRing;
use error::Error;

/// Create hardware-backed YubiHSM signer objects from the given configuration
pub fn init(_keyring: &mut KeyRing, config: &Option<YubihsmConfig>) -> Result<(), Error> {
    if config.is_none() {
        return Ok(());
    }

    panic!("YubiHSM2 support unimplemented!");
}
