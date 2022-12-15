//! Cryptographic functionality for Tendermint.
//!
//! This module provides type aliases and utility traits that facilitate
//! use of interchangeable implementations of cryptographic routines used by
//! Tendermint.
//!
//! The abstract framework enabling this extensibility is provided by the
//! `digest` and `signature` crates.

use crate::{Error, Signature};
/// Length of a SHA256 hash in bytes.
pub const HASH_SIZE: usize = 32;

/// An interface to allow digesting arbitrary byte slices.
///
/// This trait provides the most general possible interface that can be
/// implemented by host functions in popular on-chain smart contract
/// environments. As such, in can only do one-piece slice digests.
pub trait Hasher: Send + Sync {
    fn digest(data: impl AsRef<[u8]>) -> [u8; HASH_SIZE];
}

/// The default implementation of the [`Hasher`][Hashertrait] trait.
///
/// [Hashertrait]: super::Hasher
pub use sha2::Sha256;

/// A SHA256 digest implementation.
impl Hasher for Sha256 {
    fn digest(data: impl AsRef<[u8]>) -> [u8; HASH_SIZE] {
        let mut hash_bytes = [0u8; HASH_SIZE];
        hash_bytes.copy_from_slice(&data.as_ref());
        hash_bytes
    }
}

/// An interface to allow verifying signatures.
pub trait SignatureVerifier: Send + Sync {
    fn verify(&self, sign_bytes: &[u8], signature: &Signature) -> Result<(), Error>;
}
