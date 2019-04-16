//! Tendermint blockchain network information types.
//!
//! Tendermint is a high-performance blockchain consensus engine that powers
//! Byzantine fault tolerant applications written in any programming language.
//! This crate provides types for representing information about Tendermint
//! blockchain networks, including chain IDs, block IDs, and block heights.

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
    html_logo_url = "https://raw.githubusercontent.com/tendermint/kms/master/img/tendermint.png",
    html_root_url = "https://docs.rs/tendermint/0.5.0"
)]

#[cfg(feature = "amino-types")]
extern crate prost_amino as prost;
#[cfg(feature = "amino-types")]
#[macro_use]
extern crate prost_amino_derive as prost_derive;

pub mod account;
pub mod address;
pub mod algorithm;
#[cfg(feature = "amino-types")]
pub mod amino_types;
pub mod block;
pub mod chain;
pub mod error;
pub mod hash;
pub mod moniker;
pub mod node;
pub mod public_keys;
#[cfg(feature = "secret-connection")]
pub mod secret_connection;
pub mod timestamp;

#[cfg(feature = "secret-connection")]
pub use crate::secret_connection::SecretConnection;
pub use crate::{
    address::*,
    algorithm::*,
    block::{ParseHeight as ParseBlockHeight, ParseId as ParseBlockId},
    chain::ParseId as ParseChainId,
    error::Error,
    hash::*,
    moniker::Moniker,
    public_keys::*,
    timestamp::*,
};
