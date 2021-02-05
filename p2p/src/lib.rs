//! The Tendermint P2P stack.

#![forbid(unsafe_code)]
#![deny(
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms,
    nonstandard_style
)]
#![doc(
    html_root_url = "https://docs.rs/tendermint-p2p/0.1.0",
    html_logo_url = "https://raw.githubusercontent.com/informalsystems/tendermint-rs/master/img/logo-tendermint-rs_3961x4001.png"
)]

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod error;
pub mod secret_connection;
