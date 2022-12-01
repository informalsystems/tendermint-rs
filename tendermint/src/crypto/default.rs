//! Pure Rust implementations of the cryptographic traits.
//!
//! Most applications using this crate should use these implementations.
//! Alternative implementations can be useful on targets like wasm and
//! on-chain environments, where code size is at a premium and a faster
//! platform-native cryptographic API is available.

/// The default implementation of the [`Sha256`][sha256trait] trait.
///
/// [sha256trait]: super::Sha256
pub use sha2::Sha256;

/// Types implementing the ECDSA algorithm using the Secp256k1 elliptic curve.
pub mod ecdsa_secp256 {
    pub use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
}
