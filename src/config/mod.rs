//! Configuration file structures (with serde-derived parser)

use std::collections::BTreeMap;

#[cfg(feature = "dalek-provider")]
mod dalek;

#[cfg(feature = "yubihsm-provider")]
mod yubihsm;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use toml;

use error::Error;

#[cfg(feature = "dalek-provider")]
pub use self::dalek::DalekConfig;

#[cfg(feature = "yubihsm-provider")]
pub use self::yubihsm::YubihsmConfig;

#[derive(Deserialize, Debug)]
pub struct Config {
    /// Addresses of validator nodes
    pub validators: BTreeMap<String, ValidatorConfig>,

    /// Cryptographic signature provider configuration
    pub providers: ProviderConfig,

    pub secret_connection_key_path: PathBuf,
}

impl Config {
    /// Parse the configuration TOML, returning a Config struct
    pub fn load(filename: &Path) -> Result<Config, Error> {
        let mut file = File::open(filename)?;

        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        toml::from_str(&data).map_err(|e| err!(ConfigError, "parse error: {}", e))
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ValidatorConfig {
    /// Validator hostname or IP address
    pub addr: String,

    /// Validator port
    pub port: u16,

    /// Automatically reconnect on error? (default: true)
    pub reconnect: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct ProviderConfig {
    /// ed25519-dalek configuration
    #[cfg(feature = "dalek-provider")]
    pub dalek: Option<DalekConfig>,

    /// Map of yubihsm-connector labels to their configurations
    #[cfg(feature = "yubihsm-provider")]
    pub yubihsm: Option<YubihsmConfig>,
}
