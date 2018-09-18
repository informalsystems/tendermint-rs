//! Configuration for the `YubiHSM` backend

use abscissa::secrets::{DebugSecret, Secret};
use std::collections::BTreeMap;

/// The (optional) `[providers.yubihsm]` config section
pub type YubihsmConfig = BTreeMap<String, ConnectorConfig>;

/// Configuration for a particular yubihsm-connector process
#[derive(Clone, Deserialize, Debug)]
pub struct ConnectorConfig {
    /// Address of yubihsm-connector (IP or hostname)
    pub http: HttpConfig,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// Map of labels to private key configurations
    pub keys: BTreeMap<String, SigningKeyConfig>,
}

/// Configuration options for this connector
#[derive(Clone, Debug, Deserialize)]
pub struct HttpConfig {
    /// Address of the connector (IP address or DNS name)
    pub addr: String,

    /// Port the connector process is listening on
    pub port: u16,

    /// Timeout for connecting, reading, and writing in milliseconds
    pub timeout_ms: u64,
}

/// Configuration options for this connector
#[derive(Clone, Debug, Deserialize)]
pub struct AuthConfig {
    /// Authentication key ID to use to authenticate to the YubiHSM
    #[serde(rename = "key-id")]
    pub key_id: u16,

    /// Password to use to authenticate to the YubiHSM
    // TODO: allow password to be read from an external password-file
    pub password: Secret<Password>,
}

/// Password to the YubiHSM
#[derive(Clone, Deserialize)]
pub struct Password(String);

impl DebugSecret for Password {
    fn debug_secret(&self) -> &'static str {
        "[PASSWORD]"
    }
}

impl Default for Password {
    fn default() -> Self {
        Password("".to_owned())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SigningKeyConfig {
    /// Signing key ID
    #[serde(rename = "key-id")]
    pub key_id: u16,
}
