//! Configuration file structures (with serde-derived parser)

use std::collections::HashMap;

#[cfg(feature = "dalek")]
mod dalek;

#[cfg(feature = "yubihsm")]
mod yubihsm;

#[cfg(feature = "dalek")]
pub use self::dalek::DalekConfig;

#[cfg(feature = "yubihsm")]
pub use self::yubihsm::YubihsmConnectorConfig;

#[derive(Deserialize, Debug)]
pub struct Config {
    /// Addresses of validator nodes
    pub validators: HashMap<String, ValidatorConfig>,

    /// Cryptographic signature provider configuration
    pub providers: ProviderConfig,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorConfig {
    /// Validator hostname or IP address
    pub addr: String,

    /// Validator port
    pub port: u16,
}

#[derive(Deserialize, Debug)]
pub struct ProviderConfig {
    /// ed25519-dalek configuration
    #[cfg(feature = "dalek")]
    pub dalek: DalekConfig,

    /// Map of yubihsm-connector labels to their configurations
    #[cfg(feature = "yubihsm")]
    pub yubihsm: HashMap<String, YubihsmConnectorConfig>,
}
