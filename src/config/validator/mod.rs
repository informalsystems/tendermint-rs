mod addr;

use std::path::PathBuf;
use tendermint::chain;

pub use self::addr::ValidatorAddr;

/// Validator configuration
#[derive(Clone, Deserialize, Debug)]
pub struct ValidatorConfig {
    /// Address of the validator (`tcp://` or `unix://`)
    pub addr: ValidatorAddr,

    /// Chain ID of the Tendermint network this validator is part of
    pub chain_id: chain::Id,

    /// Automatically reconnect on error? (default: true)
    #[serde(default = "reconnect_default")]
    pub reconnect: bool,

    /// Path to our Ed25519 identity key (if applicable)
    pub secret_key: Option<PathBuf>,
}

/// Default value for the `ValidatorConfig` reconnect field
fn reconnect_default() -> bool {
    true
}
