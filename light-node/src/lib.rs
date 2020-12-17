//! light-node
//!
//! The Tendermint light-node wraps the light-client crate into a command-line interface tool.
//!
//! It can be used to initialize and start a standalone light client daemon and exposes a JSON-RPC
//! endpoint from which you can query the current state of the light node.

// Tip: Deny warnings with `RUSTFLAGS="-D warnings"` environment variable in CI

#![forbid(unsafe_code)]
#![warn(
    rust_2018_idioms,
    trivial_casts,
    unused_lifetimes,
    unused_qualifications
)]
#![doc(
    html_root_url = "https://docs.rs/tendermint-light-node/0.17.0",
    html_logo_url = "https://raw.githubusercontent.com/informalsystems/tendermint-rs/master/img/logo-tendermint-rs_3961x4001.png"
)]

pub mod application;
pub mod commands;
pub mod config;
pub mod error;
pub mod prelude;
pub mod rpc;
