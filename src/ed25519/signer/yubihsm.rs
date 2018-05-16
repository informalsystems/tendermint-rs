//! YubiHSM2-based signer
// TODO: finish implementing this!

use config::YubihsmConfig;
use error::Error;
use super::Signer;

/// Create hardware-backed YubiHSM signer objects from the given configuration
pub fn create_signers(
    _signers: &mut Vec<Signer>,
    config: &Option<YubihsmConfig>,
) -> Result<(), Error> {
    if config.is_none() {
        return Ok(());
    }

    panic!("YubiHSM2 support unimplemented!");
}
