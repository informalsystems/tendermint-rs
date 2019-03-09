//! Configuration for Ledger Tendermint signer

use crate::chain;

/// Ledger Tendermint signer configuration
#[derive(Clone, Deserialize, Debug)]
pub struct LedgerTendermintConfig {
    /// Chains this signing key is authorized to be used from
    pub chain_ids: Vec<chain::Id>,
}
