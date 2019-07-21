//! Remote addresses (`tcp://` or `unix://`)

use crate::node;
use failure::{bail, Error};
#[cfg(feature = "serde")]
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{self, Display},
    path::PathBuf,
    str::{self, FromStr},
};

/// Remote address (TCP or UNIX socket)
#[derive(Clone, Debug)]
pub enum Address {
    /// TCP connections
    Tcp {
        /// Remote peer ID
        peer_id: Option<node::Id>,

        /// Hostname or IP address
        host: String,

        /// Port
        port: u16,
    },

    /// UNIX domain sockets
    Unix {
        /// Path to a UNIX domain socket path
        path: PathBuf,
    },
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Address {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_str(&String::deserialize(deserializer)?)
            .map_err(|e| D::Error::custom(format!("{}", e)))
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Tcp { host, port, .. } => write!(f, "tcp://{}:{}", host, port),
            Address::Unix { path } => write!(f, "unix://{}", path.display()),
        }
    }
}

impl FromStr for Address {
    type Err = Error;

    fn from_str(addr: &str) -> Result<Self, Error> {
        if addr.starts_with("tcp://") {
            let authority_parts = addr[6..].split('@').collect::<Vec<_>>();

            let (peer_id, authority) = match authority_parts.len() {
                1 => (None, authority_parts[0]),
                2 => (Some(authority_parts[0].parse()?), authority_parts[1]),
                _ => bail!("invalid tcp:// address: {}", addr),
            };

            let host_and_port: Vec<&str> = authority.split(':').collect();

            if host_and_port.len() != 2 {
                bail!("invalid tcp:// address: {}", addr);
            }

            let host = host_and_port[0].to_owned();

            match host_and_port[1].parse() {
                Ok(port) => Ok(Address::Tcp {
                    peer_id,
                    host,
                    port,
                }),
                Err(_) => bail!("invalid tcp:// address (bad port): {}", addr),
            }
        } else if addr.starts_with("unix://") {
            Ok(Address::Unix {
                path: PathBuf::from(&addr[7..]),
            })
        } else {
            bail!("invalid address: {}", addr)
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for Address {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node;

    /// Example TCP node address
    const EXAMPLE_TCP_STR: &str =
        "tcp://abd636b766dcefb5322d8ca40011ec2cb35efbc2@35.192.61.41:26656";

    #[test]
    fn parse_tcp_addr() {
        match EXAMPLE_TCP_STR.parse::<Address>().unwrap() {
            Address::Tcp {
                peer_id,
                host,
                port,
            } => {
                assert_eq!(
                    peer_id.unwrap(),
                    "abd636b766dcefb5322d8ca40011ec2cb35efbc2"
                        .parse::<node::Id>()
                        .unwrap()
                );
                assert_eq!(host, "35.192.61.41");
                assert_eq!(port, 26656);
            }
            other => panic!("unexpected address type: {:?}", other),
        }
    }
}
