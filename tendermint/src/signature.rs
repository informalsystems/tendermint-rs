//! Cryptographic (a.k.a. digital) signatures

pub use ed25519::{Signature as Ed25519Signature, SIGNATURE_LENGTH as ED25519_SIGNATURE_SIZE};
pub use signature::{Signer, Verifier};

#[cfg(feature = "secp256k1")]
pub use k256::ecdsa::Signature as Secp256k1Signature;

use std::convert::TryFrom;
use tendermint_proto::Protobuf;

use crate::error::Error;

/// Signatures
#[derive(Clone, Debug, PartialEq)]
pub struct Signature(Vec<u8>);

impl Protobuf<Vec<u8>> for Signature {}

impl TryFrom<Vec<u8>> for Signature {
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        if bytes.is_empty() {
            return Err(Error::empty_signature());
        }

        Ok(Self(bytes))
    }
}

impl From<Signature> for Vec<u8> {
    fn from(value: Signature) -> Self {
        value.0
    }
}

impl Signature {
    /// Create a new signature from the given byte array, if non-empty.
    pub fn new(bytes: Vec<u8>) -> Option<Self> {
        if bytes.is_empty() {
            None
        } else {
            Some(Self(bytes))
        }
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
