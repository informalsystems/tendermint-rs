//! JSON-RPC requests

use super::{Id, Method, Version};
use crate::{prelude::*, Error};
use core::fmt::Debug;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// JSON-RPC requests
pub trait Request: Debug + DeserializeOwned + Serialize + Sized + Send {
    /// Response type for this command
    type Response: super::response::Response;

    /// Request method
    fn method(&self) -> Method;

    /// Serialize this request as JSON
    fn into_json(self) -> String {
        Wrapper::new(self).into_json()
    }

    /// Parse a JSON-RPC request from a JSON string.
    fn from_string(s: impl AsRef<[u8]>) -> Result<Self, Error> {
        let wrapper: Wrapper<Self> = serde_json::from_slice(s.as_ref()).map_err(Error::serde)?;
        Ok(wrapper.params)
    }
}

/// Simple JSON-RPC requests which correlate with a single response from the
/// remote endpoint.
///
/// An example of a request which is not simple would be the event subscription
/// request, which, on success, returns a [`Subscription`] and not just a
/// simple, singular response.
///
/// [`Subscription`]: struct.Subscription.html
pub trait SimpleRequest: Request {}

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
        Self::new_with_id(Id::uuid_v4(), request)
    }

    pub(crate) fn new_with_id(id: Id, request: R) -> Self {
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

    pub fn into_json(self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}
