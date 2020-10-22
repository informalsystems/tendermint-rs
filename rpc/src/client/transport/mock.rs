//! Mock client implementation for use in testing.

mod subscription;
pub use subscription::MockSubscriptionClient;

use crate::{Client, Error, Method, Request, Response, Result};
use async_trait::async_trait;
use std::collections::HashMap;

/// A mock client implementation for use in testing.
///
/// ## Examples
///
/// ```rust
/// use tendermint_rpc::{Client, Method, MockClient, MockRequestMatcher, MockRequestMethodMatcher};
///
/// const ABCI_INFO_RESPONSE: &str = r#"{
///   "jsonrpc": "2.0",
///   "id": "",
///   "result": {
///     "response": {
///       "data": "GaiaApp",
///       "version": "0.17.0",
///       "last_block_height": "488120",
///       "last_block_app_hash": "2LnCw0fN+Zq/gs5SOuya/GRHUmtWftAqAkTUuoxl4g4="
///     }
///   }
/// }"#;
///
/// #[tokio::main]
/// async fn main() {
///     let matcher = MockRequestMethodMatcher::default()
///         .map(Method::AbciInfo, Ok(ABCI_INFO_RESPONSE.to_string()));
///     let client = MockClient::new(matcher);
///
///     let abci_info = client.abci_info().await.unwrap();
///     println!("Got mock ABCI info: {:?}", abci_info);
///     assert_eq!("GaiaApp".to_string(), abci_info.data);
/// }
/// ```
#[derive(Debug)]
pub struct MockClient<M: MockRequestMatcher> {
    matcher: M,
}

#[async_trait]
impl<M: MockRequestMatcher> Client for MockClient<M> {
    async fn perform<R>(&self, request: R) -> Result<R::Response>
    where
        R: Request,
    {
        self.matcher.response_for(request).ok_or_else(|| {
            Error::client_internal_error("no matching response for incoming request")
        })?
    }
}

impl<M: MockRequestMatcher> MockClient<M> {
    /// Create a new mock RPC client using the given request matcher.
    pub fn new(matcher: M) -> Self {
        Self { matcher }
    }
}

/// A trait required by the [`MockClient`] that allows for different approaches
/// to mocking responses for specific requests.
///
/// [`MockClient`]: struct.MockClient.html
pub trait MockRequestMatcher: Send + Sync {
    /// Provide the corresponding response for the given request (if any).
    fn response_for<R>(&self, request: R) -> Option<Result<R::Response>>
    where
        R: Request;
}

/// Provides a simple [`MockRequestMatcher`] implementation that simply maps
/// requests with specific methods to responses.
///
/// [`MockRequestMatcher`]: trait.MockRequestMatcher.html
#[derive(Debug)]
pub struct MockRequestMethodMatcher {
    mappings: HashMap<Method, Result<String>>,
}

impl MockRequestMatcher for MockRequestMethodMatcher {
    fn response_for<R>(&self, request: R) -> Option<Result<R::Response>>
    where
        R: Request,
    {
        self.mappings.get(&request.method()).map(|res| match res {
            Ok(json) => R::Response::from_string(json),
            Err(e) => Err(e.clone()),
        })
    }
}

impl Default for MockRequestMethodMatcher {
    fn default() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }
}

impl MockRequestMethodMatcher {
    /// Maps all incoming requests with the given method such that their
    /// corresponding response will be `response`.
    ///
    /// Successful responses must be JSON-encoded.
    #[allow(dead_code)]
    pub fn map(mut self, method: Method, response: Result<String>) -> Self {
        self.mappings.insert(method, response);
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;
    use tendermint::block::Height;
    use tendermint::chain::Id;
    use tokio::fs;

    async fn read_json_fixture(name: &str) -> String {
        fs::read_to_string(PathBuf::from("./tests/support/").join(name.to_owned() + ".json"))
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn mock_client() {
        let abci_info_fixture = read_json_fixture("abci_info").await;
        let block_fixture = read_json_fixture("block").await;
        let matcher = MockRequestMethodMatcher::default()
            .map(Method::AbciInfo, Ok(abci_info_fixture))
            .map(Method::Block, Ok(block_fixture));
        let client = MockClient::new(matcher);

        let abci_info = client.abci_info().await.unwrap();
        assert_eq!("GaiaApp".to_string(), abci_info.data);
        assert_eq!(Height::from(488120_u32), abci_info.last_block_height);

        let block = client.block(Height::from(10_u32)).await.unwrap().block;
        assert_eq!(Height::from(10_u32), block.header().height);
        assert_eq!(
            "cosmoshub-2".parse::<Id>().unwrap(),
            block.header().chain_id
        );
    }
}
