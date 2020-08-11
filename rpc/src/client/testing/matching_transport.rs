use async_trait::async_trait;

use crate::{
    client::{testing::Fixture, transport::Transport},
    Error, Method,
};

/// A rudimentary fixture-based transport.
///
/// Fixtures, if read from the file system, are lazily evaluated.
#[derive(Debug)]
pub struct RequestMatchingTransport {
    matchers: Vec<Box<dyn RequestMatcher>>,
}

/// Implement this trait to facilitate different kinds of request matching.
pub trait RequestMatcher: Send + Sync + std::fmt::Debug {
    /// Does the given request match?
    fn matches(&self, request: &str) -> bool;

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

#[async_trait]
impl Transport for RequestMatchingTransport {
    async fn request(&self, request: String) -> Result<String, Error> {
        for matcher in &self.matchers {
            if matcher.matches(&request) {
                let response = matcher.response()?.read().await;
                return Ok(response);
            }
        }
        Err(Error::internal_error(format!(
            "no matcher for request: {}",
            request
        )))
    }
}

impl RequestMatchingTransport {
    pub fn new(matcher: impl RequestMatcher + 'static) -> Self {
        Self {
            matchers: vec![Box::new(matcher)],
        }
    }

    pub fn push(mut self, matcher: impl RequestMatcher + 'static) -> Self {
        self.matchers.push(Box::new(matcher));
        self
    }
}

impl MethodMatcher {
    pub fn new(method: Method, response: Result<Fixture, Error>) -> Self {
        Self { method, response }
    }
}

impl RequestMatcher for MethodMatcher {
    fn matches(&self, request: &str) -> bool {
        let request_json = serde_json::from_str::<serde_json::Value>(request).unwrap();
        return self.method.to_string() == request_json.get("method").unwrap().as_str().unwrap();
    }

    fn response(&self) -> Result<Fixture, Error> {
        self.response.clone()
    }
}
