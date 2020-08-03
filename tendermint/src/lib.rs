//! Tendermint is a high-performance blockchain consensus engine that powers
//! Byzantine fault tolerant applications written in any programming language.
//! This crate provides core types for representing information about Tendermint
//! blockchain networks, including chain information types, secret connections,
//! and remote procedure calls (JSONRPC).

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
    html_root_url = "https://docs.rs/tendermint/0.15.0",
    html_logo_url = "https://raw.githubusercontent.com/informalsystems/tendermint-rs/master/img/logo-tendermint-rs.png"
)]

#[macro_use]
pub mod error;

pub mod abci;
pub mod account;
pub mod amino_types;
pub mod block;
pub mod chain;
pub mod channel;
pub mod config;
pub mod consensus;
pub mod evidence;
pub mod genesis;
pub mod hash;
pub mod merkle;
mod moniker;
pub mod net;
pub mod node;
pub mod private_key;
pub mod public_key;
pub mod serializers;
pub mod signature;
pub mod time;
mod timeout;
pub mod trust_threshold;
pub mod validator;
mod version;
pub mod vote;

#[cfg(test)]
mod test;

pub use crate::genesis::Genesis;
pub use crate::{
    block::Block,
    error::{Error, Kind},
    hash::Hash,
    moniker::Moniker,
    public_key::{PublicKey, TendermintKey},
    signature::Signature,
    time::Time,
    timeout::Timeout,
    version::Version,
    vote::Vote,
};
pub use private_key::PrivateKey;
