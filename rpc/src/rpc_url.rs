//! URL representation for RPC clients.

use crate::error::Error;
use crate::prelude::*;
use core::convert::TryFrom;
use core::fmt;
use core::str::FromStr;
use serde::de::Error as SerdeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The various schemes supported by Tendermint RPC clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scheme {
    Http,
    Https,
    WebSocket,
    SecureWebSocket,
}

impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scheme::Http => write!(f, "http"),
            Scheme::Https => write!(f, "https"),
            Scheme::WebSocket => write!(f, "ws"),
            Scheme::SecureWebSocket => write!(f, "wss"),
        }
    }
}

impl FromStr for Scheme {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "http" | "tcp" => Scheme::Http,
            "https" => Scheme::Https,
            "ws" => Scheme::WebSocket,
            "wss" => Scheme::SecureWebSocket,
            _ => return Err(Error::unsupported_scheme(s.to_string())),
        })
    }
}

/// A uniform resource locator (URL), with support for only those
/// schemes/protocols supported by Tendermint RPC clients.
///
/// Re-implements relevant parts of [`url::Url`]'s interface with convenience
/// mechanisms for transformation to/from other types.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Url {
    inner: url::Url,
    scheme: Scheme,
    host: String,
    port: u16,
}

impl FromStr for Url {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner: url::Url = s.parse().map_err(Error::parse_url)?;

        let scheme: Scheme = inner.scheme().parse()?;

        let host = inner
            .host_str()
            .ok_or_else(|| Error::invalid_params(format!("URL is missing its host: {}", s)))?
            .to_owned();

        let port = inner.port_or_known_default().ok_or_else(|| {
            Error::invalid_params(format!("cannot determine appropriate port for URL: {}", s))
        })?;
        Ok(Self {
            inner,
            scheme,
            host,
            port,
        })
    }
}

impl Url {
    /// Returns whether or not this URL represents a connection to a secure
    /// endpoint.
    pub fn is_secure(&self) -> bool {
        match self.scheme {
            Scheme::Http => false,
            Scheme::Https => true,
            Scheme::WebSocket => false,
            Scheme::SecureWebSocket => true,
        }
    }

    /// Get the scheme associated with this URL.
    pub fn scheme(&self) -> Scheme {
        self.scheme
    }

    /// Get the username associated with this URL, if any.
    pub fn username(&self) -> &str {
        self.inner.username()
    }

    /// Get the password associated with this URL, if any.
    pub fn password(&self) -> Option<&str> {
        self.inner.password()
    }

    /// Get the host associated with this URL.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Get the port associated with this URL.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get this URL's path.
    pub fn path(&self) -> &str {
        self.inner.path()
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl TryFrom<url::Url> for Url {
    type Error = crate::Error;

    fn try_from(value: url::Url) -> Result<Self, Self::Error> {
        value.to_string().parse()
    }
}

impl Serialize for Url {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Url {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Url::from_str(&s).map_err(|e| D::Error::custom(e.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use lazy_static::lazy_static;

    struct ExpectedUrl {
        scheme: Scheme,
        host: String,
        port: u16,
        path: String,
        username: String,
        password: Option<String>,
    }

    lazy_static! {
        static ref SUPPORTED_URLS: Vec<(String, ExpectedUrl)> = vec![
            (
                "tcp://127.0.0.1:26657".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::Http,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "".to_string(),
                    username: "".to_string(),
                    password: None,
                }
            ),
            (
                "http://127.0.0.1:26657".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::Http,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "/".to_string(),
                    username: "".to_string(),
                    password: None,
                }
            ),
            (
                "https://127.0.0.1:26657".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::Https,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "/".to_string(),
                    username: "".to_string(),
                    password: None,
                }
            ),
            (
                "ws://127.0.0.1:26657/websocket".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::WebSocket,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "/websocket".to_string(),
                    username: "".to_string(),
                    password: None,
                }
            ),
            (
                "wss://127.0.0.1:26657/websocket".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::SecureWebSocket,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "/websocket".to_string(),
                    username: "".to_string(),
                    password: None,
                }
            )
        ];
    }

    #[test]
    fn parsing() {
        for (url_str, expected) in SUPPORTED_URLS.iter() {
            let u = Url::from_str(url_str).unwrap();
            assert_eq!(expected.scheme, u.scheme(), "{}", url_str);
            assert_eq!(expected.host, u.host(), "{}", url_str);
            assert_eq!(expected.port, u.port(), "{}", url_str);
            assert_eq!(expected.path, u.path(), "{}", url_str);
            assert_eq!(expected.username, u.username());
            if let Some(pw) = u.password() {
                assert_eq!(expected.password.as_ref().unwrap(), pw, "{}", url_str);
            } else {
                assert!(expected.password.is_none(), "{}", url_str);
            }
        }
    }
}
