//! p2p
//!
//! The Tendermint P2P stack.

// Tip: Deny warnings with `RUSTFLAGS="-D warnings"` environment variable in CI

#![forbid(unsafe_code)]
#![deny(
    warnings,
    missing_docs,
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

pub mod error;
pub mod secret_connection;
