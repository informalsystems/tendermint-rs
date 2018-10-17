/// Validator configuration
#[derive(Clone, Deserialize, Debug)]
pub struct ValidatorConfig {
    /// Validator hostname or IP address
    pub addr: String,

    /// Validator port
    pub port: u16,

    /// Automatically reconnect on error? (default: true)
    #[serde(default = "reconnect_default")]
    pub reconnect: bool,
}

impl ValidatorConfig {
    /// Get the URI which represents this configuration
    pub fn uri(&self) -> String {
        format!("gaia-rpc://{}:{}", self.addr, self.port)
    }
}

impl Default for ValidatorConfig {
    fn default() -> ValidatorConfig {
        ValidatorConfig {
            addr: "127.0.0.1".to_owned(),
            port: 26657,
            reconnect: true,
        }
    }
}

/// Default value for the `ValidatorConfig` reconnect field
fn reconnect_default() -> bool {
    ValidatorConfig::default().reconnect
}
