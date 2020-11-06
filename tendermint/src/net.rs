//! Remote addresses (`tcp://` or `unix://`)

use crate::{
    error::{Error, Kind},
    node,
};
use anomaly::{fail, format_err};

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{self, Display},
    path::PathBuf,
    str::{self, FromStr},
};

/// URI prefix for TCP connections
pub const TCP_PREFIX: &str = "tcp://";

/// URI prefix for Unix socket connections
pub const UNIX_PREFIX: &str = "unix://";

/// Remote address (TCP or UNIX socket)
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_str(&String::deserialize(deserializer)?)
            .map_err(|e| D::Error::custom(format!("{}", e)))
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Tcp { host, port, .. } => write!(f, "{}{}:{}", TCP_PREFIX, host, port),
            Address::Unix { path } => write!(f, "{}{}", UNIX_PREFIX, path.display()),
        }
    }
}

impl FromStr for Address {
    type Err = Error;

    fn from_str(addr: &str) -> Result<Self, Error> {
        if addr.starts_with(TCP_PREFIX) {
            Self::parse_tcp_addr(&addr.strip_prefix(TCP_PREFIX).unwrap())
        } else if addr.starts_with(UNIX_PREFIX) {
            Ok(Address::Unix {
                path: PathBuf::from(&addr.strip_prefix(UNIX_PREFIX).unwrap()),
            })
        } else if addr.contains("://") {
            // The only supported URI prefixes are `tcp://` and `unix://`
            fail!(Kind::Parse, "invalid address prefix: {:?}", addr)
        } else {
            // If the address has no URI prefix, assume TCP
            Self::parse_tcp_addr(addr)
        }
    }
}

impl Address {
    /// Parse a TCP address (without a `tcp://` prefix).
    ///
    /// This is used internally by `Address::from_str`.
    fn parse_tcp_addr(addr: &str) -> Result<Self, Error> {
        // TODO(tarcieri): use the `uri` (or other) crate for this
        let authority_parts = addr.split('@').collect::<Vec<_>>();

        let (peer_id, authority) = match authority_parts.len() {
            1 => (None, authority_parts[0]),
            2 => (Some(authority_parts[0].parse()?), authority_parts[1]),
            _ => fail!(
                Kind::Parse,
                "invalid {} address (bad authority): {}",
                TCP_PREFIX,
                addr
            ),
        };

        let host_and_port: Vec<&str> = authority.split(':').collect();

        if host_and_port.len() != 2 {
            fail!(
                Kind::Parse,
                "invalid {} address (missing port): {}",
                TCP_PREFIX,
                addr
            );
        }

        // TODO(tarcieri): default for missing hostname?
        let host = host_and_port[0].to_owned();

        let port = host_and_port[1].parse::<u16>().map_err(|_| {
            format_err!(
                Kind::Parse,
                "invalid {} address (bad port): {}",
                TCP_PREFIX,
                addr
            )
        })?;

        Ok(Address::Tcp {
            peer_id,
            host,
            port,
        })
    }
}

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
    const EXAMPLE_TCP_ADDR: &str =
        "tcp://abd636b766dcefb5322d8ca40011ec2cb35efbc2@35.192.61.41:26656";

    #[test]
    fn parse_tcp_addr() {
        let tcp_addr_without_prefix = &EXAMPLE_TCP_ADDR[TCP_PREFIX.len()..];

        for tcp_addr in &[EXAMPLE_TCP_ADDR, tcp_addr_without_prefix] {
            match tcp_addr.parse::<Address>().unwrap() {
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
}
