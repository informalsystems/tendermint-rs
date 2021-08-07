//! JSON-RPC response types

use crate::response_error::ResponseError;
use crate::{Error, Id, Version};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::Read;

/// JSON-RPC responses
pub trait Response: Serialize + DeserializeOwned + Sized {
    /// Parse a JSON-RPC response from a JSON string
    fn from_string(response: impl AsRef<[u8]>) -> Result<Self, Error> {
        let wrapper: Wrapper<Self> =
            serde_json::from_slice(response.as_ref()).map_err(Error::serde)?;
        wrapper.into_result()
    }

    /// Parse a JSON-RPC response from an `io::Reader`
    fn from_reader(reader: impl Read) -> Result<Self, Error> {
        let wrapper: Wrapper<Self> = serde_json::from_reader(reader).map_err(Error::serde)?;
        wrapper.into_result()
    }
}

/// JSON-RPC response wrapper (i.e. message envelope)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Wrapper<R> {
    /// JSON-RPC version
    jsonrpc: Version,

    /// Identifier included in request
    id: Id,

    /// Results of request (if successful)
    result: Option<R>,

    /// Error message if unsuccessful
    error: Option<ResponseError>,
}

impl<R> Wrapper<R>
where
    R: Response,
{
    /// Get JSON-RPC version
    pub fn version(&self) -> &Version {
        &self.jsonrpc
    }

    /// Get JSON-RPC ID
    #[allow(dead_code)]
    pub fn id(&self) -> &Id {
        &self.id
    }

    /// Convert this wrapper into the underlying error, if any
    pub fn into_error(self) -> Option<Error> {
        self.error.map(Error::response)
    }

    /// Convert this wrapper into a result type
    pub fn into_result(self) -> Result<R, Error> {
        // Ensure we're using a supported RPC version
        self.version().ensure_supported()?;

        if let Some(e) = self.error {
            Err(Error::response(e))
        } else if let Some(result) = self.result {
            Ok(result)
        } else {
            Err(Error::malformed_json())
        }
    }

    #[cfg(test)]
    pub fn new_with_id(id: Id, result: Option<R>, error: Option<ResponseError>) -> Self {
        Self {
            jsonrpc: Version::current(),
            id,
            result,
            error,
        }
    }
}
