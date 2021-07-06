//! ABCI framework for building [Tendermint] applications in Rust.
//!
//! [Tendermint]: https://tendermint.com
//!
//! ## Example
//!
//! The following example is adapted from our integration test suite to
//! demonstrate how to instantiate an ABCI application, server and client and
//! have them interact. In practice, the client interaction will be performed
//! by a full Tendermint node.
//!
//! ```rust
//! use tendermint_abci::{KeyValueStoreApp, ServerBuilder, ClientBuilder};
//! use tendermint_proto::abci::{RequestEcho, RequestDeliverTx, RequestQuery};
//!
//! // Create our key/value store application
//! let (app, driver) = KeyValueStoreApp::new();
//! // Create our server, binding it to TCP port 26658 on localhost and
//! // supplying it with our key/value store application
//! let server = ServerBuilder::default().bind("127.0.0.1:26658", app).unwrap();
//! let server_addr = server.local_addr();
//!
//! // We want the driver and the server to run in the background while we
//! // interact with them via the client in the foreground
//! std::thread::spawn(move || driver.run());
//! std::thread::spawn(move || server.listen());
//!
//! let mut client = ClientBuilder::default().connect(server_addr).unwrap();
//! let res = client
//!     .echo(RequestEcho {
//!         message: "Hello ABCI!".to_string(),
//!     })
//!     .unwrap();
//! assert_eq!(res.message, "Hello ABCI!");
//!
//! // Deliver a transaction and then commit the transaction
//! client
//!     .deliver_tx(RequestDeliverTx {
//!         tx: "test-key=test-value".as_bytes().to_owned(),
//!     })
//!     .unwrap();
//! client.commit().unwrap();
//!
//! // We should be able to query for the data we just delivered above
//! let res = client
//!     .query(RequestQuery {
//!         data: "test-key".as_bytes().to_owned(),
//!         path: "".to_string(),
//!         height: 0,
//!         prove: false,
//!     })
//!     .unwrap();
//! assert_eq!(res.value, "test-value".as_bytes().to_owned());
//! ```

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
