//! URL representation for RPC clients.

use crate::{Error, Result};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

/// The various schemes supported by Tendermint RPC clients.
#[derive(Debug, Clone, Copy)]
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
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            // TODO(thane): Remove the tcp scheme once we've refactored net::Address.
            "http" | "tcp" => Scheme::Http,
            "https" => Scheme::Https,
            "ws" => Scheme::WebSocket,
            "wss" => Scheme::SecureWebSocket,
            _ => return Err(Error::invalid_params(&format!("unsupported scheme: {}", s))),
        })
    }
}

/// A uniform resource locator (URL), with support for only those
/// schemes/protocols supported by Tendermint RPC clients.
///
/// Re-implements relevant parts of [`url::Url`]'s interface with convenience
/// mechanisms for transformation to/from other types.
#[derive(Debug, Clone)]
pub struct Url {
    inner: url::Url,
    scheme: Scheme,
    host: String,
    port: u16,
}

impl FromStr for Url {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let inner: url::Url = s.parse()?;
        let scheme: Scheme = inner.scheme().parse()?;
        let host = inner
            .host_str()
            .ok_or_else(|| Error::invalid_params(&format!("URL is missing its host: {}", s)))?
            .to_owned();
        let port = inner.port_or_known_default().ok_or_else(|| {
            Error::invalid_params(&format!("cannot determine appropriate port for URL: {}", s))
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
    type Error = Error;

    fn try_from(value: url::Url) -> Result<Self> {
        value.to_string().parse()
    }
}
