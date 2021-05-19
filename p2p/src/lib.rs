//! The Tendermint P2P stack.

#![forbid(unsafe_code)]
#![deny(
    nonstandard_style,
    private_in_public,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![warn(
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    clippy::pedantic,
    clippy::unwrap_used,
    missing_docs,
    unused_import_braces,
    unused_qualifications
)]
#![doc(
    html_root_url = "https://docs.rs/tendermint-p2p/0.19.0",
    html_logo_url = "https://raw.githubusercontent.com/informalsystems/tendermint-rs/master/img/logo-tendermint-rs_3961x4001.png"
)]
// TODO(xla): Temporary to suppress noisy warnings.
#![allow(dead_code)]

pub mod error;
pub mod secret_connection;

mod message;
mod peer;
mod supervisor;
mod transport;
