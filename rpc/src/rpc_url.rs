//! URL representation for RPC clients.

use core::{convert::TryFrom, fmt, str::FromStr};

use serde::{de::Error as SerdeError, Deserialize, Deserializer, Serialize, Serializer};

use crate::{error::Error, prelude::*};

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
}

impl FromStr for Url {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url: url::Url = s.parse().map_err(Error::parse_url)?;
        url.try_into()
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
    pub fn username(&self) -> Option<&str> {
        Some(self.inner.username()).filter(|s| !s.is_empty())
    }

    /// Get the password associated with this URL, if any.
    pub fn password(&self) -> Option<&str> {
        self.inner.password()
    }

    /// Get the authority associated with this URL, if any.
    /// The authority is the username and password separated by a colon.
    pub fn authority(&self) -> Option<String> {
        self.username()
            .map(|user| format!("{}:{}", user, self.password().unwrap_or_default()))
    }

    /// Get the host associated with this URL.
    pub fn host(&self) -> &str {
        self.inner.host_str().unwrap()
    }

    /// Get the port associated with this URL.
    pub fn port(&self) -> u16 {
        self.inner.port_or_known_default().unwrap()
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

impl AsRef<url::Url> for Url {
    fn as_ref(&self) -> &url::Url {
        &self.inner
    }
}

impl From<Url> for url::Url {
    fn from(value: Url) -> Self {
        value.inner
    }
}

impl TryFrom<url::Url> for Url {
    type Error = crate::Error;

    fn try_from(url: url::Url) -> Result<Self, Self::Error> {
        let scheme: Scheme = url.scheme().parse()?;

        if url.host_str().is_none() {
            return Err(Error::invalid_params(format!(
                "URL is missing its host: {url}"
            )));
        }

        if url.port_or_known_default().is_none() {
            return Err(Error::invalid_params(format!(
                "cannot determine appropriate port for URL: {url}"
            )));
        }

        Ok(Self { inner: url, scheme })
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
    use lazy_static::lazy_static;

    use super::*;

    struct ExpectedUrl {
        scheme: Scheme,
        host: String,
        port: u16,
        path: String,
        username: Option<String>,
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
                    username: None,
                    password: None,
                }
            ),
            (
                "tcp://foo@127.0.0.1:26657".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::Http,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "".to_string(),
                    username: Some("foo".to_string()),
                    password: None,
                }
            ),
            (
                "tcp://foo:bar@127.0.0.1:26657".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::Http,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "".to_string(),
                    username: Some("foo".to_string()),
                    password: Some("bar".to_string()),
                }
            ),
            (
                "http://127.0.0.1:26657".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::Http,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "/".to_string(),
                    username: None,
                    password: None,
                }
            ),
            (
                "http://foo@127.0.0.1:26657".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::Http,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "/".to_string(),
                    username: Some("foo".to_string()),
                    password: None,
                }
            ),
            (
                "http://foo:bar@127.0.0.1:26657".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::Http,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "/".to_string(),
                    username: Some("foo".to_string()),
                    password: Some("bar".to_string()),
                }
            ),
            (
                "https://127.0.0.1:26657".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::Https,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "/".to_string(),
                    username: None,
                    password: None,
                }
            ),
            (
                "https://foo@127.0.0.1:26657".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::Https,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "/".to_string(),
                    username: Some("foo".to_string()),
                    password: None,
                }
            ),
            (
                "https://foo:bar@127.0.0.1:26657".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::Https,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "/".to_string(),
                    username: Some("foo".to_string()),
                    password: Some("bar".to_string()),
                }
            ),
            (
                "ws://127.0.0.1:26657/websocket".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::WebSocket,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "/websocket".to_string(),
                    username: None,
                    password: None,
                }
            ),
            (
                "ws://foo@127.0.0.1:26657/websocket".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::WebSocket,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "/websocket".to_string(),
                    username: Some("foo".to_string()),
                    password: None,
                }
            ),
            (
                "ws://foo:bar@127.0.0.1:26657/websocket".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::WebSocket,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "/websocket".to_string(),
                    username: Some("foo".to_string()),
                    password: Some("bar".to_string()),
                }
            ),
            (
                "wss://127.0.0.1:26657/websocket".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::SecureWebSocket,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "/websocket".to_string(),
                    username: None,
                    password: None,
                }
            ),
            (
                "wss://foo@127.0.0.1:26657/websocket".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::SecureWebSocket,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "/websocket".to_string(),
                    username: Some("foo".to_string()),
                    password: None,
                }
            ),
            (
                "wss://foo:bar@127.0.0.1:26657/websocket".to_owned(),
                ExpectedUrl {
                    scheme: Scheme::SecureWebSocket,
                    host: "127.0.0.1".to_string(),
                    port: 26657,
                    path: "/websocket".to_string(),
                    username: Some("foo".to_string()),
                    password: Some("bar".to_string()),
                }
            )
        ];
    }

    #[test]
    fn parsing() {
        for (url_str, expected) in SUPPORTED_URLS.iter() {
            let u = Url::from_str(url_str).unwrap();
            assert_eq!(expected.scheme, u.scheme(), "{url_str}");
            assert_eq!(expected.host, u.host(), "{url_str}");
            assert_eq!(expected.port, u.port(), "{url_str}");
            assert_eq!(expected.path, u.path(), "{url_str}");
            if let Some(n) = u.username() {
                assert_eq!(expected.username.as_ref().unwrap(), n, "{url_str}");
            } else {
                assert!(expected.username.is_none(), "{}", url_str);
            }
            if let Some(pw) = u.password() {
                assert_eq!(expected.password.as_ref().unwrap(), pw, "{url_str}");
            } else {
                assert!(expected.password.is_none(), "{}", url_str);
            }
        }
    }
}
