//! JSON-RPC requests

use core::fmt::Debug;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::{Id, Method, Version};
use crate::dialect::{DefaultDialect, Dialect};
use crate::{prelude::*, Error};

/// JSON-RPC requests
pub trait Request<S: Dialect>: DeserializeOwned + Serialize + Sized + Send {
    /// Response type for this command
    type Response: super::response::Response;

    /// Request method
    fn method(&self) -> Method;

    /// Serialize this request as JSON
    fn into_json(self) -> String {
        Wrapper::new_with_dialect(S::default(), self).into_json()
    }

    /// Parse a JSON-RPC request from a JSON string.
    fn from_string(s: impl AsRef<[u8]>) -> Result<Self, Error> {
        let wrapper: Wrapper<Self, S> = serde_json::from_slice(s.as_ref()).map_err(Error::serde)?;
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
pub trait SimpleRequest<S: Dialect>: Request<S> {}

/// JSON-RPC request wrapper (i.e. message envelope)
#[derive(Debug, Deserialize, Serialize)]
pub struct Wrapper<R, S> {
    /// JSON-RPC version
    jsonrpc: Version,

    /// Identifier included in request
    id: Id,

    /// Request method
    method: Method,

    /// Request parameters (i.e. request object)
    params: R,

    /// Dialect tag
    #[serde(skip)]
    #[allow(dead_code)]
    dialect: S,
}

impl<R> Wrapper<R, DefaultDialect>
where
    R: Request<DefaultDialect>,
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
            dialect: Default::default(),
        }
    }
}

impl<R, S> Wrapper<R, S>
where
    R: Request<S>,
    S: Dialect,
{
    /// Create a new request wrapper from the given request.
    ///
    /// The ID of the request is set to a random [UUIDv4] value.
    ///
    /// [UUIDv4]: https://en.wikipedia.org/wiki/Universally_unique_identifier#Version_4_(random)
    pub fn new_with_dialect(dialect: S, request: R) -> Self {
        Self::new_with_id_and_dialect(Id::uuid_v4(), dialect, request)
    }

    pub(crate) fn new_with_id_and_dialect(id: Id, dialect: S, request: R) -> Self {
        Self {
            jsonrpc: Version::current(),
            id,
            method: request.method(),
            params: request,
            dialect,
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
