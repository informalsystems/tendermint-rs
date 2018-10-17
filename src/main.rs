//! Tendermint Key Management System

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
extern crate hkdf;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prost_derive;
extern crate rand;
extern crate ring;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate sha2;
extern crate signatory;
extern crate signatory_dalek;
#[cfg(feature = "yubihsm")]
extern crate signatory_yubihsm;
extern crate subtle_encoding;
extern crate x25519_dalek;

#[macro_use]
mod error;

mod application;
mod client;
mod commands;
mod config;
mod ed25519;
mod rpc;
mod secret_connection;
mod session;
mod types;
#[cfg(feature = "yubihsm")]
mod yubihsm;

use application::KmsApplication;

/// Main entry point
fn main() {
    abscissa::boot(KmsApplication);
}
