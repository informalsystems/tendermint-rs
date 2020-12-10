//! Cryptographic (a.k.a. digital) signatures

pub use ed25519::{Signature as Ed25519Signature, SIGNATURE_LENGTH as ED25519_SIGNATURE_SIZE};
pub use signature::{Signer, Verifier};

#[cfg(feature = "secp256k1")]
pub use k256::ecdsa::Signature as Secp256k1;

use crate::{Error, Kind};
use std::convert::TryFrom;
use tendermint_proto::Protobuf;

/// Signatures
#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Signature {
    /// Ed25519 block signature
    Ed25519(Ed25519Signature),
    /// No signature present
    None, /* This could have been implemented as an `Option<>` but then handling it would be
           * outside the scope of this enum. */
}

impl Protobuf<Vec<u8>> for Signature {}

impl TryFrom<Vec<u8>> for Signature {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(Self::default());
        }
        if value.len() != ED25519_SIGNATURE_SIZE {
            return Err(Kind::InvalidSignatureIdLength.into());
        }
        let mut slice: [u8; ED25519_SIGNATURE_SIZE] = [0; ED25519_SIGNATURE_SIZE];
        slice.copy_from_slice(&value[..]);
        Ok(Signature::Ed25519(Ed25519Signature::new(slice)))
    }
}

impl From<Signature> for Vec<u8> {
    fn from(value: Signature) -> Self {
        value.as_bytes().to_vec()
    }
}

impl Default for Signature {
    fn default() -> Self {
        Signature::None
    }
}

impl Signature {
    /// Return the algorithm used to create this particular signature
    pub fn algorithm(&self) -> Algorithm {
        match self {
            Signature::Ed25519(_) => Algorithm::Ed25519,
            Signature::None => Algorithm::Ed25519, /* It doesn't matter what algorithm an empty
                                                    * signature has. */
        }
    }

    /// Get Ed25519 signature
    pub fn ed25519(self) -> Option<Ed25519Signature> {
        match self {
            Signature::Ed25519(sig) => Some(sig),
            Signature::None => None,
        }
    }

    /// Return the raw bytes of this signature
    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    /// Get a vector containing the byte serialization of this key
    pub fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        match self {
            Signature::Ed25519(sig) => sig.as_ref(),
            Signature::None => &[],
        }
    }
}

impl From<Ed25519Signature> for Signature {
    fn from(pk: Ed25519Signature) -> Signature {
        Signature::Ed25519(pk)
    }
}

/// Digital signature algorithms
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Algorithm {
    /// ECDSA over secp256k1
    EcdsaSecp256k1,

    /// EdDSA over Curve25519
    Ed25519,
}
