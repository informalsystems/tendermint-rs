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
extern crate abscissa_derive;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

#[macro_use]
mod error;

mod application;
mod client;
mod commands;
mod config;
mod keyring;
mod rpc;
mod session;
mod unix_connection;
#[cfg(feature = "yubihsm")]
mod yubihsm;

pub use crate::{application::KmsApplication, unix_connection::UnixConnection};
