//! Cryptographic functionality for Tendermint.
//!
//! This module provides type aliases and utility traits that facilitate
//! use of interchangeable implementations of cryptographic routines used by
//! Tendermint.
//!
//! The abstract framework enabling this extensibility is provided by the
//! `digest` and `signature` crates.

pub mod sha256;
pub use sha256::Sha256;

#[cfg(feature = "rust-crypto")]
pub mod default;
