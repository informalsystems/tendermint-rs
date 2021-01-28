//! ABCI framework for building applications with Tendermint.

mod application;
#[cfg(feature = "client")]
pub mod client;
pub mod codec;
mod result;
pub mod runtime;
pub mod server;

// Example applications
#[cfg(feature = "echo-app")]
pub use application::echo::EchoApp;

// Common exports
pub use application::Application;
pub use result::{Error, Result};

// Runtime-specific convenience exports
#[cfg(all(feature = "blocking", feature = "runtime-std"))]
pub use server::blocking::StdServerBuilder;
#[cfg(all(feature = "non-blocking", feature = "runtime-async-std"))]
pub use server::non_blocking::AsyncStdServerBuilder;
#[cfg(all(feature = "non-blocking", feature = "runtime-tokio"))]
pub use server::non_blocking::TokioServerBuilder;

#[cfg(feature = "client")]
mod client_exports {
    #[cfg(all(feature = "blocking", feature = "runtime-std"))]
    pub use super::client::blocking::StdClientBuilder;
    #[cfg(all(feature = "non-blocking", feature = "runtime-async-std"))]
    pub use super::client::non_blocking::AsyncStdClientBuilder;
    #[cfg(all(feature = "non-blocking", feature = "runtime-tokio"))]
    pub use super::client::non_blocking::TokioClientBuilder;
}
#[cfg(feature = "client")]
pub use client_exports::*;
