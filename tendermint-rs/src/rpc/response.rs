//! JSONRPC response types

use super::{Error, Id, Version};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// JSONRPC responses
pub trait Response: Serialize + DeserializeOwned + Sized {
    /// Parse a JSONRPC response from a JSON string
    fn from_json(response: &str) -> Result<Self, Error> {
        let wrapper: Wrapper<Self> = serde_json::from_str(response).map_err(Error::parse_error)?;
        wrapper.into_result()
    }
}

/// JSONRPC response wrapper (i.e. message envelope)
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum Wrapper<R> {
    /// RPC completed successfully
    Success { jsonrpc: Version, id: Id, result: R },

    /// RPC error
    Error {
        jsonrpc: Version,
        id: Id,
        error: Error,
    },
}

impl<R> Wrapper<R> {
    /// Get JSONRPC version
    pub fn version(&self) -> &Version {
        match self {
            Wrapper::Success { jsonrpc, .. } => jsonrpc,
            Wrapper::Error { jsonrpc, .. } => jsonrpc,
        }
    }

    /// Get JSONRPC ID
    #[allow(dead_code)]
    pub fn id(&self) -> &Id {
        match self {
            Wrapper::Success { id, .. } => id,
            Wrapper::Error { id, .. } => id,
        }
    }

    /// Convert this wrapper into a result type
    pub fn into_result(self) -> Result<R, Error> {
        // Ensure we're using a supported RPC version
        self.version().ensure_supported()?;

        match self {
            Wrapper::Success { result, .. } => Ok(result),
            Wrapper::Error { error, .. } => Err(error),
        }
    }
}
