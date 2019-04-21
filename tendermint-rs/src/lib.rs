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
    html_root_url = "https://docs.rs/tendermint/0.6.0"
)]

#[cfg(feature = "amino-types")]
extern crate prost_amino as prost;
#[cfg(feature = "amino-types")]
#[macro_use]
extern crate prost_amino_derive as prost_derive;

pub mod account;
#[cfg(feature = "amino-types")]
pub mod amino_types;
pub mod block;
pub mod chain;
#[cfg(feature = "rpc")]
pub mod channel;
pub mod consensus;
pub mod error;
pub mod evidence;
#[cfg(feature = "rpc")]
pub mod genesis;
pub mod hash;
mod moniker;
pub mod net;
pub mod node;
pub mod public_key;
#[cfg(feature = "rpc")]
pub mod rpc;
#[cfg(feature = "secret-connection")]
pub mod secret_connection;
#[cfg(feature = "serde")]
mod serializers;
pub mod signature;
pub mod time;
pub mod transaction;
pub mod validator;
mod version;
pub mod vote;

#[cfg(feature = "rpc")]
pub use crate::genesis::Genesis;
#[cfg(feature = "secret-connection")]
pub use crate::secret_connection::SecretConnection;
pub use crate::{
    block::Block,
    error::Error,
    hash::Hash,
    moniker::Moniker,
    public_key::{PublicKey, TendermintKey},
    signature::Signature,
    time::Time,
    version::Version,
    vote::Vote,
};
