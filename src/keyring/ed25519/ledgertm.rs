//! Ledger Tendermint signer

use crate::{
    chain,
    config::provider::ledgertm::LedgerTendermintConfig,
    error::{KmsError, KmsErrorKind::*},
    keyring::{ed25519::Signer, SigningProvider},
};
use ledger_tendermint::signer::Ed25519LedgerTmAppSigner;
use signatory::PublicKeyed;
use tendermint::TendermintKey;

/// Create Ledger Tendermint signer object from the given configuration
pub fn init(
    chain_registry: &mut chain::Registry,
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

    let provider = Ed25519LedgerTmAppSigner::connect()?;
    // TODO(tarcieri): support for adding account keys into keyrings
    let public_key = TendermintKey::ConsensusKey(provider.public_key()?.into());
    let signer = Signer::new(SigningProvider::LedgerTm, public_key, Box::new(provider));

    for chain_id in &ledgertm_configs[0].chain_ids {
        chain_registry.add_to_keyring(chain_id, signer.clone())?;
    }

    Ok(())
}
