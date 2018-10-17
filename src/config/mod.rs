//! Configuration file structures (with serde-derived parser)

use std::path::PathBuf;

pub mod provider;
mod validator;

use self::provider::ProviderConfig;
pub use self::validator::*;

/// Name of the KMS configuration file
pub const CONFIG_FILE_NAME: &str = "tmkms.toml";

/// KMS configuration (i.e. TOML file parsed with serde)
#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct KmsConfig {
    /// Addresses of validator nodes
    pub validator: Vec<ValidatorConfig>,

    /// Cryptographic signature provider configuration
    pub providers: ProviderConfig,

    /// Secret connection configuration
    #[serde(rename = "secret-connection")]
    pub secret_connection: SecretConnectionConfig,
}

// Impl the `abscissa::GlobalConfig` trait, storing the configuration in the
// `GLOBAL_CONFIG` static value
impl_global_config!(KmsConfig, GLOBAL_CONFIG);

#[derive(Clone, Deserialize, Debug)]
pub struct SecretConnectionConfig {
    /// Path to our identity key
    #[serde(rename = "secret-key-path")]
    pub secret_key_path: PathBuf,
}
