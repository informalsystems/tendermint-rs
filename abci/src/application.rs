//! `async` ABCI server application interface.

#[cfg(feature = "echo-app")]
pub mod echo;

use async_trait::async_trait;
use tendermint::abci::{request, response};

#[async_trait]
pub trait Application: Send + Clone {
    /// Request that the ABCI server echo back the same message sent to it.
    fn echo(&self, req: request::Echo) -> response::Echo {
        response::Echo::new(req.message)
    }

    /// Receive information about the Tendermint node and respond with
    /// information about the ABCI application.
    fn info(&self, _req: request::Info) -> response::Info {
        Default::default()
    }
}
