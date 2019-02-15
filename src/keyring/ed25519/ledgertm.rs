//! Ledger Tendermint signer

use signatory::PublicKeyed;
use signatory_ledger_tm::{self, Ed25519LedgerTmAppSigner};

use crate::{
    error::KmsError,
    config::provider::ledgertm::LedgerTendermintConfig,
    keyring::{ed25519::Signer, KeyRing},
};

pub const LEDGER_TM_PROVIDER_LABEL: &str = "ledgertm";
pub const LEDGER_TM_ID: &str = "ledgertm";

/// Create Ledger Tendermint signer object from the given configuration
pub fn init(keyring: &mut KeyRing, _config: &[LedgerTendermintConfig]) -> Result<(), KmsError> {
    let provider = Box::new(Ed25519LedgerTmAppSigner::connect()?);
    let pk = provider.public_key()?;
    let signer = Signer::new(LEDGER_TM_PROVIDER_LABEL, LEDGER_TM_ID.to_string(), provider);
    keyring.add(pk, signer)?;
    Ok(())
}
