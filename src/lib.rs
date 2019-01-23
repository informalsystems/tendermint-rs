//! Tendermint Key Management System

#![crate_name = "tmkms"]
#![crate_type = "rlib"]
#![deny(
    warnings,
    missing_docs,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]

extern crate prost_amino as prost;
#[macro_use]
extern crate abscissa;
#[macro_use]
extern crate abscissa_derive;
extern crate bytes;
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sha2;
extern crate signal_hook;
extern crate signatory;
extern crate signatory_dalek;
#[cfg(feature = "yubihsm")]
extern crate signatory_yubihsm;
extern crate subtle_encoding;
extern crate tendermint;

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

pub use crate::application::KmsApplication;
pub use crate::unix_connection::UnixConnection;
