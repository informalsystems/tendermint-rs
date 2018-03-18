//! Configuration file structures (with serde-derived parser)

use std::collections::HashMap;

#[cfg(feature = "dalek")]
mod dalek;

#[cfg(feature = "yubihsm")]
mod yubihsm;

use std::fs::File;
use std::io::Read;
use toml;

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

impl Config {
    /// Parse the configuration TOML, returning a Config struct
    pub fn load(filename: &str) -> Config {
        let mut file =
            File::open(filename).unwrap_or_else(|e| panic!("couldn't open {}: {}", filename, e));

        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        toml::from_str(&data).unwrap_or_else(|e| panic!("couldn't parse {}: {:?}", filename, e))
    }
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
