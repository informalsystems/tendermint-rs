//! Validator addresses (`tcp://` or `unix://`)

use crate::error::{KmsError, KmsErrorKind::*};
use serde::{de::Error as DeError, Deserialize, Deserializer};
use std::{
    fmt::{self, Display},
    path::PathBuf,
    str::{self, FromStr},
};
use tendermint::secret_connection;

#[derive(Clone, Debug)]
pub enum ValidatorAddr {
    /// TCP connections (with SecretConnection transport encryption)
    Tcp {
        /// Remote peer ID of the validator
        // TODO(tarcieri): make this mandatory
        peer_id: Option<secret_connection::PeerId>,

        /// Validator hostname or IP address
        host: String,

        /// Validator port
        port: u16,
    },

    /// UNIX domain sockets
    Unix { socket_path: PathBuf },
}

impl ValidatorAddr {
    /// Get the URI representation of this configuration
    pub fn to_uri(&self) -> String {
        self.to_string()
    }
}

impl<'de> Deserialize<'de> for ValidatorAddr {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_str(&String::deserialize(deserializer)?)
            .map_err(|e| D::Error::custom(format!("{}", e)))
    }
}

impl Display for ValidatorAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidatorAddr::Tcp { host, port, .. } => write!(f, "tcp://{}:{}", host, port),
            ValidatorAddr::Unix { socket_path } => write!(f, "unix://{}", socket_path.display()),
        }
    }
}

impl FromStr for ValidatorAddr {
    type Err = KmsError;

    // TODO: less janky URL parser? (e.g. use `url` crate)
    fn from_str(addr: &str) -> Result<Self, KmsError> {
        if addr.starts_with("tcp://") {
            let authority_parts = addr[6..].split('@').collect::<Vec<_>>();
            let (peer_id, authority) = match authority_parts.len() {
                1 => (None, authority_parts[0]),
                2 => (Some(authority_parts[0].parse()?), authority_parts[1]),
                _ => fail!(ConfigError, "invalid tcp:// address: {}", addr),
            };

            let host_and_port: Vec<&str> = authority.split(':').collect();

            if host_and_port.len() != 2 {
                fail!(ConfigError, "invalid tcp:// address: {}", addr);
            }

            let host = host_and_port[0].to_owned();
            let port = host_and_port[1]
                .parse()
                .map_err(|_| err!(ConfigError, "invalid tcp:// address (bad port): {}", addr))?;

            Ok(ValidatorAddr::Tcp {
                peer_id,
                host,
                port,
            })
        } else if addr.starts_with("unix://") {
            let socket_path = PathBuf::from(&addr[7..]);
            Ok(ValidatorAddr::Unix { socket_path })
        } else {
            fail!(ConfigError, "invalid addr: {}", addr)
        }
    }
}
