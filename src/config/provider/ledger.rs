//! Configuration for ledger-backed signer

/// Ledger signer configuration
#[derive(Clone, Deserialize, Debug)]
pub struct LedgerConfig {
    pub active: bool,
}
