//! Cryptographic (a.k.a. digital) signatures

pub use ed25519::Signature as Ed25519Signature;
#[cfg(feature = "secp256k1")]
pub use k256::ecdsa::Signature as Secp256k1Signature;

use bytes::Bytes;

use tendermint_proto::Protobuf;

use crate::{error::Error, prelude::*};

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

impl TryFrom<Bytes> for Signature {
    type Error = Error;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        Self::new_non_empty(bytes)
    }
}

impl From<Signature> for Bytes {
    fn from(value: Signature) -> Self {
        value.0.into()
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
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<Ed25519Signature> for Signature {
    fn from(sig: Ed25519Signature) -> Signature {
        Self(sig.to_vec())
    }
}

#[cfg(feature = "rust-crypto")]
impl From<ed25519_consensus::Signature> for Signature {
    fn from(sig: ed25519_consensus::Signature) -> Signature {
        Self(sig.to_bytes().to_vec())
    }
}

#[cfg(feature = "secp256k1")]
impl From<Secp256k1Signature> for Signature {
    fn from(sig: Secp256k1Signature) -> Signature {
        Self(sig.to_vec())
    }
}
