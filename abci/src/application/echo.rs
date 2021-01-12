//! Trivial ABCI application that just implements echo functionality.

use crate::Application;

/// Trivial ABCI application that just implements echo functionality.
#[derive(Clone)]
pub struct EchoApp {}

impl EchoApp {
    /// Constructor.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for EchoApp {
    fn default() -> Self {
        EchoApp::new()
    }
}

impl Application for EchoApp {}
