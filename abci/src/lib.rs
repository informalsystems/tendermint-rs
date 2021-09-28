//! ABCI framework for building [Tendermint] applications in Rust.
//!
//! [Tendermint]: https://tendermint.com

mod application;
#[cfg(feature = "client")]
mod client;
mod codec;
pub mod error;
mod server;

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
