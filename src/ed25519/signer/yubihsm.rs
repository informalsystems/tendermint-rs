//! YubiHSM2-based signer
// TODO: finish implementing this!

use super::Signer;
use config::YubihsmConfig;
use error::Error;

/// Create hardware-backed YubiHSM signer objects from the given configuration
// TODO: return an iterator rather than taking an `&mut Vec<Signer>`?
pub fn create_signers(
    _signers: &mut Vec<Signer>,
    config: &Option<YubihsmConfig>,
) -> Result<(), Error> {
    if config.is_none() {
        return Ok(());
    }

    panic!("YubiHSM2 support unimplemented!");
}
