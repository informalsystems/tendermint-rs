//! Connection configuration

use std::path::PathBuf;

#[derive(Debug)]
pub enum ConnectionConfig {
    /// A secret connection config kind
    SecretConnection(SecretConnectionConfig),

    /// A UNIX connection config kind
    UNIXConnection(UNIXConnectionConfig),
}

#[derive(Clone, Deserialize, Debug)]
pub struct SecretConnectionConfig {
    /// Path to our identity key
    #[serde(rename = "secret-key-path")]
    pub secret_key_path: PathBuf,

    /// Validator hostname or IP address
    pub addr: String,

    /// Validator port
    pub port: u16,
}

#[derive(Clone, Deserialize, Debug)]
pub struct UNIXConnectionConfig {
    /// A UNIX socket path
    #[serde(rename = "socket-path")]
    pub socket_path: PathBuf,
}

impl SecretConnectionConfig {
    pub fn uri(&self) -> String {
        format!("gaia-rpc://{}:{}", self.addr, self.port)
    }
}

impl UNIXConnectionConfig {
    pub fn uri(&self) -> String {
        format!("gaia-ipc://{}", self.socket_path.to_str().unwrap())
    }
}

impl ConnectionConfig {
    /// Get the URI which represents this configuration
    pub fn uri(&self) -> String {
        match *self {
            ConnectionConfig::SecretConnection(ref conf) => conf.uri(),
            ConnectionConfig::UNIXConnection(ref conf) => conf.uri(),
        }
    }
}

impl Default for SecretConnectionConfig {
    fn default() -> Self {
        Self {
            secret_key_path: PathBuf::from(r"/path/to/secret-key"),
            addr: "127.0.0.1".to_owned(),
            port: 26657,
        }
    }
}

impl Default for UNIXConnectionConfig {
    fn default() -> Self {
        Self {
            socket_path: PathBuf::from(r"/tmp/cosmos-kms.sock"),
        }
    }
}
