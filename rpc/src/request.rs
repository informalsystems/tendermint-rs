//! JSONRPC requests

use super::{Id, Method, Version};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

/// JSONRPC requests
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

/// JSONRPC request wrapper (i.e. message envelope)
#[derive(Debug, Deserialize, Serialize)]
pub struct Wrapper<R> {
    /// JSONRPC version
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
    /// By default this sets the ID of the request to a random [UUIDv4] value.
    ///
    /// [UUIDv4]: https://en.wikipedia.org/wiki/Universally_unique_identifier
    pub fn new(request: R) -> Self {
        Wrapper::new_with_id(Id::uuid_v4(), request)
    }

    /// Create a new request wrapper with a custom JSONRPC request ID.
    pub fn new_with_id(id: Id, request: R) -> Self {
        Self {
            jsonrpc: Version::current(),
            id,
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
