//! Tendermint Key Management System

#![deny(
    warnings,
    missing_docs,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]

#[cfg(not(any(feature = "softsign", feature = "yubihsm", feature = "ledgertm")))]
compile_error!(
    "please enable one of the following backends with cargo's --features argument: \
     yubihsm, ledgertm, softsign (e.g. --features=yubihsm)"
);

extern crate prost_amino as prost;
#[macro_use]
extern crate abscissa;
#[macro_use]
extern crate log;

pub mod application;
pub mod chain;
pub mod client;
pub mod commands;
pub mod config;
pub mod error;
pub mod keyring;
pub mod prelude;
pub mod rpc;
pub mod session;
pub mod unix_connection;
#[cfg(feature = "yubihsm")]
pub mod yubihsm;

pub use crate::{application::KmsApplication, unix_connection::UnixConnection};
