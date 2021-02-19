//! Trivial ABCI echo application

use crate::Application;

/// Trivial echo application, mainly for testing purposes.
#[derive(Clone)]
pub struct EchoApp;

impl Default for EchoApp {
    fn default() -> Self {
        Self {}
    }
}

impl Application for EchoApp {}
