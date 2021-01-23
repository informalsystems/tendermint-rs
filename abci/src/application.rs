//! ABCI application-related types.

#[cfg(feature = "echo-app")]
pub mod echo;

use tendermint::abci::{request, response};

/// ABCI server application interface.
pub trait Application: Send + Clone + 'static {
    /// Request that the ABCI server echo back the same message sent to it.
    fn echo(&self, req: request::Echo) -> response::Echo {
        response::Echo {
            message: req.message,
        }
    }

    /// Receive information about the Tendermint node and respond with
    /// information about the ABCI application.
    fn info(&self, _req: request::Info) -> response::Info {
        Default::default()
    }

    /// Generic handler for mapping incoming requests to responses.
    fn handle(&self, req: request::Request) -> response::Response {
        match req {
            request::Request::Echo(echo) => response::Response::Echo(self.echo(echo)),
            request::Request::Info(info) => response::Response::Info(self.info(info)),
        }
    }
}
