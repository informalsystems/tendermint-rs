//! ABCI clients for interacting with ABCI servers.

#[cfg(feature = "with-async-std")]
pub mod async_std;
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

    /// Provide information to the ABCI server about the Tendermint node in
    /// exchange for information about the application.
    async fn info(&mut self, req: request::Info) -> Result<response::Info> {
        self.perform(req).await
    }

    /// Generic method to perform the given [`Request`].
    async fn perform<R: RequestInner>(&mut self, req: R) -> Result<R::Response>;
}
