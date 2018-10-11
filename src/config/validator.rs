use config::connection::{UNIXConnectionConfig, SecretConnectionConfig};

/// Validator configuration
#[derive(Clone, Deserialize, Debug)]
pub struct ValidatorConfig {
    /// Secret Connection config
    pub seccon: Option<SecretConnectionConfig>,

    /// UNIX socket config
    pub unix: Option<UNIXConnectionConfig>,

    /// Automatically reconnect on error? (default: true)
    #[serde(default = "reconnect_default")]
    pub reconnect: bool,
}

/// Default value for the `ValidatorConfig` reconnect field
fn reconnect_default() -> bool {
    true
}
