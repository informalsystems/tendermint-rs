//! This module defines the `Authorization` type for
//! authorizing a HTTP or WebSocket RPC client using
//! HTTP Basic authentication.

use alloc::string::{String, ToString};
use core::fmt;

use subtle_encoding::base64;
use url::Url;

/// An HTTP authorization.
///
/// Currently only HTTP Basic authentication is supported.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Authorization {
    Basic(String),
}

impl fmt::Display for Authorization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Basic(cred) => write!(f, "Basic {cred}"),
        }
    }
}

/// Extract the authorization, if any, from the authority part of the given URI.
///
/// This authorization can then be supplied to the RPC server via
/// the `Authorization` HTTP header.
pub fn authorize(url: &Url) -> Option<Authorization> {
    let authority = url.authority();

    if let Some((userpass, _)) = authority.split_once('@') {
        let bytes = base64::encode(userpass);
        let credentials = String::from_utf8_lossy(bytes.as_slice());
        Some(Authorization::Basic(credentials.to_string()))
    } else {
        None
    }
}

pub fn strip_authority(url: Url) -> Url {
    let mut stripped = url;
    stripped.set_username("").unwrap();
    stripped.set_password(None).unwrap();
    stripped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_auth_absent() {
        let uri = "http://example.com".parse().unwrap();
        assert_eq!(authorize(&uri), None);
    }

    #[test]
    fn extract_auth_username_only() {
        let uri = "http://toto@example.com".parse().unwrap();
        let base64 = "dG90bw==".to_string();
        assert_eq!(authorize(&uri), Some(Authorization::Basic(base64)));
    }

    #[test]
    fn extract_auth_username_password() {
        let uri = "http://toto:tata@example.com".parse().unwrap();
        let base64 = "dG90bzp0YXRh".to_string();
        assert_eq!(authorize(&uri), Some(Authorization::Basic(base64)));
    }

    #[test]
    fn strip_authority_absent() {
        let uri = "http://example.com".parse().unwrap();
        assert_eq!(strip_authority(uri), "http://example.com".parse().unwrap());
    }

    #[test]
    fn strip_auth_username_password() {
        let uri = "http://toto:tata@example.com".parse().unwrap();
        assert_eq!(strip_authority(uri), "http://example.com".parse().unwrap());
    }
}
