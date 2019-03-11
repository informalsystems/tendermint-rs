//! Configuration file structures (with serde-derived parser)

pub mod chain;
pub mod provider;
pub mod validator;

pub use self::validator::*;
use self::{chain::ChainConfig, provider::ProviderConfig};

/// Environment variable containing path to config file
pub const CONFIG_ENV_VAR: &str = "TMKMS_CONFIG_FILE";

/// Name of the KMS configuration file
pub const CONFIG_FILE_NAME: &str = "tmkms.toml";

/// KMS configuration (i.e. TOML file parsed with serde)
#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct KmsConfig {
    /// Chains the KMS is providing key management service for
    #[serde(default)]
    pub chain: Vec<ChainConfig>,

    /// Addresses of validator nodes
    #[serde(default)]
    pub validator: Vec<ValidatorConfig>,

    /// Cryptographic signature provider configuration
    pub providers: ProviderConfig,
}

// Impl the `abscissa::GlobalConfig` trait, storing the configuration in the
// `GLOBAL_CONFIG` static value
impl_global_config!(KmsConfig, GLOBAL_CONFIG);
