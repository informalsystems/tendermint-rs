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
extern crate byteorder;
extern crate bytes;
extern crate chrono;
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prost_amino_derive as prost_derive;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate iq_bech32;
extern crate serde_json;
extern crate sha2;
extern crate signatory;
extern crate signatory_dalek;
#[cfg(feature = "yubihsm")]
extern crate signatory_yubihsm;
extern crate subtle_encoding;
extern crate tm_secret_connection;

#[macro_use]
mod error;

mod application;
mod client;
mod commands;
mod config;
mod ed25519;
mod rpc;
mod session;
mod types;
#[cfg(feature = "yubihsm")]
mod yubihsm;

pub use application::KmsApplication;
