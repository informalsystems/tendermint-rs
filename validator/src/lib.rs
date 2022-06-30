//! A framework for building key management solutions for [Tendermint] validators in Rust.
//!
//! [Tendermint]: https://tendermint.com
pub mod config;
pub mod error;
pub mod server;
pub mod signer;
pub mod state;

pub use config::{BasicServerConfig, GrpcSocket};
pub use server::KMSServer;
pub use signer::{SignerProvider, SoftwareSigner};
pub use state::{FileStateProvider, ValidatorStateProvider};
