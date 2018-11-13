//! Tendermint blockchain network information types.
//!
//! Tendermint is a high-performance blockchain consensus engine that powers
//! Byzantine fault tolerant applications written in any programming language.
//! This crate provides types for representing information about Tendermint
//! blockchain networks, including chain IDs, block IDs, and block heights.

#![crate_name = "tendermint"]
#![crate_type = "rlib"]
#![deny(
    warnings,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![forbid(unsafe_code)]
#![doc(
    html_logo_url = "https://pbs.twimg.com/profile_images/909728874560233472/cb9rWojF_400x400.jpg",
    html_root_url = "https://docs.rs/tendermint/0.1.0"
)]

#[cfg(feature = "secret-connection")]
extern crate byteorder;
extern crate bytes;
extern crate chrono;
pub extern crate digest;
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[cfg(feature = "secret-connection")]
extern crate prost_amino as prost;
#[cfg(feature = "secret-connection")]
#[macro_use]
extern crate prost_amino_derive as prost_derive;
#[cfg(feature = "serializers")]
extern crate serde;
#[cfg(feature = "serializers")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "secret-connection")]
pub extern crate sha2;
#[cfg(feature = "secret-connection")]
pub extern crate signatory;
#[cfg(feature = "secret-connection")]
extern crate signatory_dalek;
extern crate subtle_encoding;
#[cfg(feature = "tai64")]
extern crate tai64;
#[cfg(feature = "secret-connection")]
extern crate zeroize;

pub mod algorithm;
pub mod amino_types;
pub mod block;
pub mod chain;
pub mod error;
pub mod hash;
pub mod public_keys;
#[cfg(feature = "secret-connection")]
pub mod secret_connection;
pub mod timestamp;

pub use algorithm::*;
pub use block::{ParseHeight as ParseBlockHeight, ParseId as ParseBlockId};
pub use chain::ParseId as ParseChainId;
pub use error::*;
pub use hash::*;
pub use public_keys::*;
#[cfg(feature = "secret-connection")]
pub use secret_connection::SecretConnection;
pub use timestamp::*;
