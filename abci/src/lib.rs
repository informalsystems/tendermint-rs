//! ABCI framework for building Tendermint applications in Rust.

mod application;
#[cfg(feature = "client")]
mod client;
mod codec;
mod result;
mod server;

// Common exports
pub use application::Application;
#[cfg(feature = "client")]
pub use client::{Client, ClientBuilder};
pub use result::{Error, Result};
pub use server::{Server, ServerBuilder};

// Example applications
#[cfg(feature = "echo-app")]
pub use application::echo::EchoApp;
