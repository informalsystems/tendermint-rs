//! `async` ABCI server application interface.

use async_trait::async_trait;
use tendermint::abci::{request, response};

#[async_trait]
pub trait Application: Send + Clone {
    /// Request that the ABCI server echo back the same message sent to it.
    fn echo(&self, request: request::Echo) -> response::Echo {
        response::Echo::new(request.message)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    /// Simple echo application for use in testing.
    #[derive(Clone)]
    pub struct EchoApp {}

    impl Default for EchoApp {
        fn default() -> Self {
            Self {}
        }
    }

    impl Application for EchoApp {}
}
