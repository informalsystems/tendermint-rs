//! JSONRPC response types

use failure::{format_err, Error};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::{self, Display};

/// JSONRPC responses
pub trait Response: Serialize + DeserializeOwned + Sized {
    /// Parse a JSONRPC response from a JSON string
    fn from_json(response: &str) -> Result<Self, Error> {
        let wrapper: ResponseWrapper<Self> =
            serde_json::from_str(response).map_err(|e| format_err!("error parsing JSON: {}", e))?;

        // TODO(tarcieri): check JSONRPC version/ID?
        Ok(wrapper.result)
    }
}

/// JSONRPC response wrapper (i.e. message envelope)
#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseWrapper<R> {
    /// JSONRPC version
    pub jsonrpc: Version,

    /// ID
    pub id: Id,

    /// Result
    pub result: R,
}

/// JSONRPC version
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Version(String);

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// JSONRPC ID
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Id(String);

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
