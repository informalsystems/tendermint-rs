//! ABCI framework for building Tendermint applications in Rust.
//!
//! ## Example
//!
//! The following example shows how to use the trivial [`EchoApp`] to
//! instantiate an ABCI server.
//!
//! **NOTE**: `EchoApp` won't work when coupled with a real Tendermint node, as
//! it does not implement the minimum ABCI application functionality required
//! by Tendermint. See the [`KeyValueStoreApp`] for a more functional ABCI
//! application.
//!
//! ```rust
//! use tendermint_abci::{EchoApp, ServerBuilder, ClientBuilder};
//! use tendermint_proto::abci::RequestEcho;
//!
//! let server = ServerBuilder::default()
//!     .bind("127.0.0.1:26658", EchoApp::default())
//!     .unwrap();
//! let server_addr = server.local_addr();
//! std::thread::spawn(move || server.listen().unwrap());
//!
//! let mut client = ClientBuilder::default()
//!     .connect(server_addr)
//!     .unwrap();
//!
//! let message = String::from("Hello ABCI!");
//! let response = client.echo(RequestEcho { message: message.clone() }).unwrap();
//! assert_eq!(response.message, message);
//! ```

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
