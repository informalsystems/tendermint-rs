//! Ledger-based signer

use signatory::PublicKeyed;
use signatory_ledger_tm::Ed25519LedgerTmAppSigner;

use crate::{
    config::provider::ledger::LedgerConfig,
    error::KmsError,
    keyring::{ed25519::Signer, KeyRing},
};

/// Label for ed25519-dalek provider
// TODO: use a non-string type for these, e.g. an enum
pub const LEDGER_PROVIDER_LABEL: &str = "ledger";

// TODO: Maybe make this depend on the app. This may not matter since the Ledger doesn't hold multiple keys. Could work with HD deriv path.
pub const LEDGER_ID: &str = "1";

/// Create hardware-backed Ledger signer object from the given configuration
pub fn init(keyring: &mut KeyRing, _ledger_configs: &[LedgerConfig]) -> Result<(), KmsError> {
    // TODO: Maybe use the active field from the config.
    let provider = Ed25519LedgerTmAppSigner::connect().unwrap();
    keyring.add(
        provider.public_key().unwrap(),
        Signer::new(
            LEDGER_PROVIDER_LABEL,
            LEDGER_ID.to_owned(),
            Box::new(provider),
        ),
    )?;
    Ok(())
}
