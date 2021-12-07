//! Trivial ABCI echo application

use crate::Application;

/// Trivial echo application, mainly for testing purposes.
#[derive(Clone, Default)]
pub struct EchoApp;

impl Application for EchoApp {}
