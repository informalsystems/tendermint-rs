//! Pure Rust implementations of the cryptographic traits.
//!
//! Most applications using this crate should use these implementations.
//! Alternative implementations can be useful on targets like wasm and
//! on-chain environments, where code size is at a premium and a faster
//! platform-native cryptographic API is available.

use super::sha256::HASH_SIZE;
use digest::Digest;

/// The default implementation of the [`Sha256`][sha256trait] trait.
///
/// [sha256trait]: super::Sha256
pub use sha2::Sha256;

impl super::Sha256 for Sha256 {
    fn digest(data: impl AsRef<[u8]>) -> [u8; HASH_SIZE] {
        <Self as Digest>::digest(data).into()
    }
}

pub mod signature;

/// Types implementing the ECDSA algorithm using the Secp256k1 elliptic curve.
#[cfg(feature = "secp256k1")]
pub mod ecdsa_secp256k1 {
    pub use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
}
