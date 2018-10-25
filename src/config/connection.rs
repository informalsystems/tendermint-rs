//! Connection configuration

use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ConnectionConfig {
    /// TCP connections (with SecretConnection transport encryption)
    #[serde(rename = "tcp")]
    Tcp {
        /// Validator hostname or IP address
        addr: String,

        /// Validator port
        port: u16,

        /// Path to our Ed25519 identity key
        secret_key_path: PathBuf,
    },

    /// UNIX domain sockets
    #[serde(rename = "unix")]
    Unix { socket_path: PathBuf },
}

impl ConnectionConfig {
    /// Get the URI representation of this configuration
    pub fn uri(&self) -> String {
        match self {
            ConnectionConfig::Tcp { addr, port, .. } => format!("gaia-rpc://{}:{}", addr, port),
            ConnectionConfig::Unix { socket_path } => {
                format!("gaia-ipc://{}", socket_path.display())
            }
        }
    }
}
