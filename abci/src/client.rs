//! ABCI clients for interacting with ABCI servers.

#[cfg(feature = "with-tokio")]
pub mod tokio;

use crate::Result;
use async_trait::async_trait;
use tendermint::abci::request::RequestInner;
use tendermint::abci::{request, response};

/// An asynchronous ABCI client.
#[async_trait]
pub trait Client {
    /// Request that the ABCI server echo back the message in the given
    /// request.
    async fn echo(&mut self, req: request::Echo) -> Result<response::Echo> {
        self.perform(req).await
    }

    /// Generic method to perform the given request.
    async fn perform<R: RequestInner>(&mut self, req: R) -> Result<R::Response>;
}
