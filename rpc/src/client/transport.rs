//! Transport layer abstraction for the Tendermint RPC client.

use async_trait::async_trait;

use crate::Error;

pub mod http_ws;

/// Transport layer abstraction that allows us to easily simulate interactions
/// with remote Tendermint nodes' RPC endpoints.
#[async_trait]
pub trait Transport: std::fmt::Debug {
    /// Perform a request to the remote endpoint, expecting a response.
    async fn request(&self, request: String) -> Result<String, Error>;
}
