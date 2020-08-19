//! Cryptographic (a.k.a. digital) signatures

pub use ed25519::{Signature as Ed25519, SIGNATURE_LENGTH as ED25519_SIGNATURE_SIZE};
pub use signature::{Signer, Verifier};

#[cfg(feature = "secp256k1")]
pub use k256::ecdsa::Signature as Secp256k1;

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use signature::Signature as _;
use subtle_encoding::base64;

/// Signatures
#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Signature {
    /// Ed25519 block signature
    Ed25519(Ed25519),
}

impl Signature {
    /// Return the algorithm used to create this particular signature
    pub fn algorithm(&self) -> Algorithm {
        match self {
            Signature::Ed25519(_) => Algorithm::Ed25519,
        }
    }

    /// Get Ed25519 signature
    pub fn ed25519(self) -> Option<Ed25519> {
        match self {
            Signature::Ed25519(sig) => Some(sig),
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
        }
    }
}

impl From<Ed25519> for Signature {
    fn from(pk: Ed25519) -> Signature {
        Signature::Ed25519(pk)
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = base64::decode(String::deserialize(deserializer)?.as_bytes())
            .map_err(D::Error::custom)?;

        Ed25519::from_bytes(&bytes)
            .map(Into::into)
            .map_err(D::Error::custom)
    }
}

impl Serialize for Signature {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        String::from_utf8(base64::encode(self.as_ref()))
            .unwrap()
            .serialize(serializer)
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
