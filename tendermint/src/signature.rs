//! Cryptographic (a.k.a. digital) signatures

pub use ed25519::Signature as Ed25519Signature;
pub use signature::{Signer, Verifier};

#[cfg(feature = "secp256k1")]
pub use k256::ecdsa::Signature as Secp256k1Signature;

use crate::prelude::*;
use core::convert::TryFrom;
use tendermint_proto::Protobuf;

use crate::error::Error;

#[deprecated(since = "0.23.2", note = "use Ed25519Signature::BYTE_SIZE instead")]
pub const ED25519_SIGNATURE_SIZE: usize = Ed25519Signature::BYTE_SIZE;

/// The expected length of all currently supported signatures, in bytes.
pub const SIGNATURE_LENGTH: usize = 64;

/// Signatures
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature(Vec<u8>);

impl Protobuf<Vec<u8>> for Signature {}

impl TryFrom<Vec<u8>> for Signature {
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::new_non_empty(bytes)
    }
}

impl TryFrom<&[u8]> for Signature {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::new_non_empty(bytes)
    }
}

impl From<Signature> for Vec<u8> {
    fn from(value: Signature) -> Self {
        value.0
    }
}

impl Signature {
    /// Create a new signature from the given byte array, if non-empty.
    ///
    /// If the given byte array is empty, returns `Ok(None)`.
    pub fn new<B: AsRef<[u8]>>(bytes: B) -> Result<Option<Self>, Error> {
        let bytes = bytes.as_ref();
        if bytes.is_empty() {
            return Ok(None);
        }
        if bytes.len() != SIGNATURE_LENGTH {
            return Err(Error::signature_invalid(format!(
                "expected signature to be {} bytes long, but was {} bytes",
                SIGNATURE_LENGTH,
                bytes.len()
            )));
        }

        Ok(Some(Self(bytes.to_vec())))
    }

    fn new_non_empty<B: AsRef<[u8]>>(bytes: B) -> Result<Self, Error> {
        Self::new(bytes)?.ok_or_else(Error::empty_signature)
    }

    /// Return a reference to the underlying byte array
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    /// Return the underlying byte array
    pub fn to_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<Ed25519Signature> for Signature {
    fn from(pk: Ed25519Signature) -> Signature {
        Self(pk.as_ref().to_vec())
    }
}

#[cfg(feature = "secp256k1")]
impl From<Secp256k1Signature> for Signature {
    fn from(pk: Secp256k1Signature) -> Signature {
        Self(pk.as_ref().to_vec())
    }
}
