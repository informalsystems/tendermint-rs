//! JSONRPC requests

use super::{Id, Method, Version};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

/// JSONRPC requests
pub trait Request: Debug + DeserializeOwned + Serialize + Sized {
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
struct Wrapper<R> {
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
    /// Create a new request wrapper from the given request
    pub fn new(request: R) -> Self {
        Self {
            jsonrpc: Version::current(),
            id: Id::uuid_v4(),
            method: request.method(),
            params: request,
        }
    }
}
