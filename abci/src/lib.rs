//! ABCI framework for building applications with Tendermint.

mod application;
mod codec;
mod result;
mod server;

// Client exports
#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::Client;

#[cfg(all(feature = "client", feature = "with-tokio"))]
pub use client::tokio::TokioClient;

// Server exports
#[cfg(feature = "with-tokio")]
pub use server::tokio::TokioServer;

// Common exports
pub use application::Application;
pub use result::{Error, Result};
