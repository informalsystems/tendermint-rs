//! This module defines the `Authorization` type for
//! authorizing a HTTP or WebSocket RPC client using
//! HTTP Basic authentication.

use alloc::string::{String, ToString};
use core::fmt;

use http::Uri;
use subtle_encoding::base64;

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
pub fn authorize(uri: &Uri) -> Option<Authorization> {
    let authority = uri.authority()?;

    if let Some((userpass, _)) = authority.as_str().split_once('@') {
        let bytes = base64::encode(userpass);
        let credentials = String::from_utf8_lossy(bytes.as_slice());
        Some(Authorization::Basic(credentials.to_string()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use http::Uri;

    use super::*;

    #[test]
    fn extract_auth_absent() {
        let uri = Uri::from_str("http://example.com").unwrap();
        assert_eq!(authorize(&uri), None);
    }

    #[test]
    fn extract_auth_username_only() {
        let uri = Uri::from_str("http://toto@example.com").unwrap();
        let base64 = "dG90bw==".to_string();
        assert_eq!(authorize(&uri), Some(Authorization::Basic(base64)));
    }

    #[test]
    fn extract_auth_username_password() {
        let uri = Uri::from_str("http://toto:tata@example.com").unwrap();
        let base64 = "dG90bzp0YXRh".to_string();
        assert_eq!(authorize(&uri), Some(Authorization::Basic(base64)));
    }
}
