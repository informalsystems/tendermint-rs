//! Key Management System for Cosmos Validators

#[macro_use]
extern crate abscissa;
#[macro_use]
extern crate abscissa_derive;
extern crate byteorder;
extern crate bytes;
extern crate chrono;
extern crate clear_on_drop;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate hex;
extern crate hkdf;
#[macro_use]
extern crate lazy_static;
extern crate prost;
#[macro_use]
extern crate prost_derive;
extern crate rand;
extern crate ring;
#[macro_use]
extern crate serde_derive;
extern crate sha2;
extern crate signatory;
#[macro_use]
extern crate serde_json;
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

use application::KMSApplication;

/// Main entry point
fn main() {
    abscissa::boot(KMSApplication);
}
