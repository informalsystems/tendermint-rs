//! JSONRPC response types

use super::{Error, Id, Version};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::Read;

/// JSONRPC responses
pub trait Response: Serialize + DeserializeOwned + Sized {
    /// Parse a JSONRPC response from a JSON string
    fn from_string(response: impl AsRef<[u8]>) -> Result<Self, Error> {
        let wrapper: Wrapper<Self> =
            serde_json::from_slice(response.as_ref()).map_err(Error::parse_error)?;
        wrapper.into_result()
    }

    /// Parse a JSONRPC response from an `io::Reader`
    fn from_reader(reader: impl Read) -> Result<Self, Error> {
        let wrapper: Wrapper<Self> = serde_json::from_reader(reader).map_err(Error::parse_error)?;
        wrapper.into_result()
    }
}

/// JSONRPC response wrapper (i.e. message envelope)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Wrapper<R> {
    /// JSONRPC version
    jsonrpc: Version,

    /// Identifier included in request
    id: Id,

    /// Results of request (if successful)
    result: Option<R>,

    /// Error message if unsuccessful
    error: Option<Error>,
}

impl<R> Wrapper<R>
where
    R: Response,
{
    /// Get JSONRPC version
    pub fn version(&self) -> &Version {
        &self.jsonrpc
    }

    /// Get JSONRPC ID
    #[allow(dead_code)]
    pub fn id(&self) -> &Id {
        &self.id
    }

    /// Convert this wrapper into a result type
    pub fn into_result(self) -> Result<R, Error> {
        // Ensure we're using a supported RPC version
        self.version().ensure_supported()?;

        if let Some(error) = self.error {
            Err(error)
        } else if let Some(result) = self.result {
            Ok(result)
        } else {
            Err(Error::server_error(
                "server returned malformatted JSON (no 'result' or 'error')",
            ))
        }
    }
}
