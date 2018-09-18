//! Configuration file structures (with serde-derived parser)

use std::collections::BTreeMap;
use std::path::PathBuf;

mod dalek;
pub use self::dalek::DalekConfig;

#[cfg(feature = "yubihsm-provider")]
mod yubihsm;
#[cfg(feature = "yubihsm-provider")]
pub use self::yubihsm::YubihsmConfig;

/// Name of the KMS configuration file
pub const CONFIG_FILE_NAME: &str = "kms.toml";

/// KMS configuration (i.e. TOML file parsed with serde)
#[derive(Clone, Deserialize, Debug)]
pub struct KMSConfig {
    /// Addresses of validator nodes
    pub validators: BTreeMap<String, ValidatorConfig>,

    /// Cryptographic signature provider configuration
    pub providers: ProviderConfig,

    /// Secret connection configuration
    #[serde(rename = "secret-connection")]
    pub secret_connection: SecretConnectionConfig,
}

// Impl the `abscissa::GlobalConfig` trait, storing the configuration in the
// `GLOBAL_CONFIG` static value
impl_global_config!(KMSConfig, GLOBAL_CONFIG);

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

#[derive(Clone, Deserialize, Debug)]
pub struct ProviderConfig {
    /// ed25519-dalek configuration
    pub dalek: Option<DalekConfig>,

    /// Map of yubihsm-connector labels to their configurations
    #[cfg(feature = "yubihsm-provider")]
    pub yubihsm: Option<YubihsmConfig>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct SecretConnectionConfig {
    /// Path to our identity key
    #[serde(rename = "secret-key-path")]
    pub secret_key_path: PathBuf,
}

/// Default value for the `ValidatorConfig` reconnect field
fn reconnect_default() -> bool {
    true
}
