//! ABCI framework for building Tendermint applications in Rust.

mod application;
#[cfg(feature = "client")]
mod client;
mod codec;
mod error;
mod server;

// Re-exported
pub use eyre::Result;

// Common exports
pub use application::Application;
#[cfg(feature = "client")]
pub use client::{Client, ClientBuilder};
pub use error::Error;
pub use server::{Server, ServerBuilder};

// Example applications
#[cfg(feature = "echo-app")]
pub use application::echo::EchoApp;
#[cfg(feature = "kvstore-app")]
pub use application::kvstore::{KeyValueStoreApp, KeyValueStoreDriver};
