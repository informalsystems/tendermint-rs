//! ABCI framework for building applications with Tendermint.

mod application;
#[cfg(any(
    feature = "client",
    feature = "runtime-tokio",
    feature = "runtime-async-std"
))]
mod codec;
mod result;
pub mod runtime;
mod server;

// Client exports
#[cfg(feature = "client")]
mod client;

#[cfg(feature = "client")]
pub use client::Client;

#[cfg(all(feature = "client", feature = "runtime-async-std"))]
pub use client::async_std::AsyncStdClient;
#[cfg(all(feature = "client", feature = "runtime-tokio"))]
pub use client::tokio::TokioClient;

// Server exports
#[cfg(feature = "runtime-async-std")]
pub use server::async_std::AsyncStdServer;
#[cfg(feature = "runtime-tokio")]
pub use server::tokio::TokioServer;

// Example applications
#[cfg(feature = "echo-app")]
pub use application::echo::EchoApp;

// Common exports
pub use application::Application;
pub use result::{Error, Result};
