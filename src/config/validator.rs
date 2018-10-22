use config::connection::ConnectionConfig;
use tendermint::chain;

/// Validator configuration
#[derive(Clone, Deserialize, Debug)]
pub struct ValidatorConfig {
    /// Chain ID for this validator
    pub chain_id: chain::Id,

    /// Connection configuration
    pub connection: ConnectionConfig,

    /// Automatically reconnect on error? (default: true)
    #[serde(default = "reconnect_default")]
    pub reconnect: bool,
}

/// Default value for the `ValidatorConfig` reconnect field
fn reconnect_default() -> bool {
    true
}
