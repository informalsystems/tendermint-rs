//! Configuration file structures (with serde-derived parser)

mod connection;
pub mod provider;
mod validator;

pub use self::connection::*;
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
}

// Impl the `abscissa::GlobalConfig` trait, storing the configuration in the
// `GLOBAL_CONFIG` static value
impl_global_config!(KmsConfig, GLOBAL_CONFIG);
