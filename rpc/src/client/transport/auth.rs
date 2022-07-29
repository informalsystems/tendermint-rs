//! This module defines the `Authorization` type for
//! authorizing a HTTP or WebSocket RPC client using
//! HTTP Basic authentication.

use alloc::string::{String, ToString};
use core::fmt;

use http::Uri;
use subtle_encoding::base64;

/// An HTTP authorization.
///
/// Currenlty only HTTP Basic authentication is supported.
#[derive(Clone, Debug)]
pub enum Authorization {
    Basic(String),
}

impl fmt::Display for Authorization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Basic(cred) => write!(f, "Basic {}", cred),
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
