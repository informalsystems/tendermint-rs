use crate::client::test_support::Fixture;
use crate::client::transport::{ClosableTransport, Transport};
use crate::{Error, Method, Request, Response};
use async_trait::async_trait;

/// A rudimentary fixture-based transport.
///
/// Fixtures, if read from the file system, are lazily evaluated.
#[derive(Debug)]
pub struct RequestMatchingTransport<M: RequestMatcher> {
    matchers: Vec<M>,
}

#[async_trait]
impl<M: RequestMatcher> Transport for RequestMatchingTransport<M> {
    // This transport does not facilitate any subscription mechanism.
    type SubscriptionTransport = ();

    async fn request<R>(&self, request: R) -> Result<R::Response, Error>
    where
        R: Request,
    {
        for matcher in &self.matchers {
            if matcher.matches(&request) {
                let response_json = matcher.response()?.read().await;
                return R::Response::from_string(response_json);
            }
        }
        Err(Error::internal_error(format!(
            "no matcher for request: {:?}",
            request
        )))
    }

    async fn subscription_transport(&self) -> Result<Self::SubscriptionTransport, Error> {
        unimplemented!()
    }
}

#[async_trait]
impl<M: RequestMatcher> ClosableTransport for RequestMatchingTransport<M> {
    async fn close(self) -> Result<(), Error> {
        Ok(())
    }
}

impl<M: RequestMatcher> RequestMatchingTransport<M> {
    pub fn new(matcher: M) -> Self {
        Self {
            matchers: vec![matcher],
        }
    }

    pub fn push(mut self, matcher: M) -> Self {
        self.matchers.push(matcher);
        self
    }
}

/// Implement this trait to facilitate different kinds of request matching.
pub trait RequestMatcher: Send + Sync + std::fmt::Debug {
    /// Does the given request match?
    fn matches<R>(&self, request: &R) -> bool
    where
        R: Request;

    /// The response we need to return if the request matches.
    fn response(&self) -> Result<Fixture, Error>;
}

/// A simple matcher that just returns a specific response every time it gets
/// a request of a particular request method.
#[derive(Debug)]
pub struct MethodMatcher {
    method: Method,
    response: Result<Fixture, Error>,
}

impl MethodMatcher {
    pub fn new(method: Method, response: Result<Fixture, Error>) -> Self {
        Self { method, response }
    }
}

impl RequestMatcher for MethodMatcher {
    fn matches<R>(&self, request: &R) -> bool
    where
        R: Request,
    {
        return self.method == request.method();
    }

    fn response(&self) -> Result<Fixture, Error> {
        self.response.clone()
    }
}
