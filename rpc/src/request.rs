//! JSON-RPC requests

use super::{Id, Method, Version};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

/// JSON-RPC requests
pub trait Request: Debug + DeserializeOwned + Serialize + Sized + Send {
    /// Response type for this command
    type Response: super::response::Response;

    /// Request method
    fn method(&self) -> Method;

    /// Serialize this request as JSON
    fn into_json(self) -> String {
        serde_json::to_string_pretty(&Wrapper::new(self)).unwrap()
    }
}

/// JSON-RPC request wrapper (i.e. message envelope)
#[derive(Debug, Deserialize, Serialize)]
pub struct Wrapper<R> {
    /// JSON-RPC version
    jsonrpc: Version,

    /// Identifier included in request
    id: Id,

    /// Request method
    method: Method,

    /// Request parameters (i.e. request object)
    params: R,
}

impl<R> Wrapper<R>
where
    R: Request,
{
    /// Create a new request wrapper from the given request.
    ///
    /// The ID of the request is set to a random [UUIDv4] value.
    ///
    /// [UUIDv4]: https://en.wikipedia.org/wiki/Universally_unique_identifier#Version_4_(random)
    pub fn new(request: R) -> Self {
        Self {
            jsonrpc: Version::current(),
            // NB: The WebSocket client relies on this being some kind of UUID,
            // and will break if it's not.
            id: Id::uuid_v4(),
            method: request.method(),
            params: request,
        }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn params(&self) -> &R {
        &self.params
    }
}
