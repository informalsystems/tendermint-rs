//! JSONRPC versions

use super::error::Error;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Supported JSONRPC version
pub const SUPPORTED_VERSION: &str = "2.0";

/// JSONRPC version
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Version(String);

impl Version {
    /// Is this JSONRPC version supported?
    pub fn is_supported(&self) -> bool {
        self.0.eq(SUPPORTED_VERSION)
    }

    /// Ensure we have a supported version or return an error
    pub fn ensure_supported(&self) -> Result<(), Error> {
        if self.is_supported() {
            Ok(())
        } else {
            Err(Error::unsupported_version(self))
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
