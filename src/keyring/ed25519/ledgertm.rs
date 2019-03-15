//! Ledger Tendermint signer

use crate::{
    config::provider::ledgertm::LedgerTendermintConfig,
    error::{KmsError, KmsErrorKind::*},
    keyring::{ed25519::Signer, KeyRing, SigningProvider},
};
use signatory::PublicKeyed;
use tendermint::TendermintKey;
use ledger_tendermint::signer::Ed25519LedgerTmAppSigner;

/// Create Ledger Tendermint signer object from the given configuration
pub fn init(
    keyring: &mut KeyRing,
    ledgertm_configs: &[LedgerTendermintConfig],
) -> Result<(), KmsError> {
    if ledgertm_configs.is_empty() {
        return Ok(());
    }

    if ledgertm_configs.len() != 1 {
        fail!(
            ConfigError,
            "expected one [providers.ledgertm] in config, found: {}",
            ledgertm_configs.len()
        );
    }

    let provider = Box::new(Ed25519LedgerTmAppSigner::connect()?);

    // TODO(tarcieri): support for adding account keys into keyrings
    let public_key = TendermintKey::ConsensusKey(provider.public_key()?.into());

    let signer = Signer::new(
        SigningProvider::LedgerTm,
        &ledgertm_configs[0].chain_ids,
        provider,
    );

    keyring.add(public_key, signer)?;

    Ok(())
}
