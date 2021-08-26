//! Cryptographic (a.k.a. digital) signatures

pub use ed25519::{Signature as Ed25519Signature, SIGNATURE_LENGTH as ED25519_SIGNATURE_SIZE};
pub use signature::{Signer, Verifier};

#[cfg(feature = "secp256k1")]
pub use k256::ecdsa::Signature as Secp256k1Signature;

use crate::error::Error;
use std::convert::TryFrom;
use tendermint_proto::Protobuf;

/// Signatures
#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Signature {
    /// Ed25519 block signature
    Ed25519(Ed25519Signature),

    /// Secp256k1 signature
    #[cfg(feature = "secp256k1")]
    #[cfg_attr(docsrs, doc(cfg(feature = "secp256k1")))]
    Secp256k1(Secp256k1Signature),

    /// No signature present
    None, /* This could have been implemented as an `Option<>` but then handling it would be
           * outside the scope of this enum. */
}

impl Protobuf<Vec<u8>> for Signature {}

impl TryFrom<Vec<u8>> for Signature {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

impl TryFrom<&'_ [u8]> for Signature {
    type Error = Error;

    fn try_from(value: &'_ [u8]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(Self::None);
        }

        let sig = Self::from_raw_ed25519(value);

        #[cfg(feature = "secp256k1")]
        let sig = sig.or_else(|_| Self::from_raw_secp256k1(value));

        sig.map_err(|_| {
            Error::signature_invalid("malformed Ed25519 or Secp256k1 signature".to_string())
        })
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
    /// Parse an Ed25519 signature from raw bytes
    fn from_raw_ed25519(value: &[u8]) -> Result<Self, Error> {
        if value.len() != ED25519_SIGNATURE_SIZE {
            return Err(Error::invalid_signature_id_length());
        }

        let mut slice: [u8; ED25519_SIGNATURE_SIZE] = [0; ED25519_SIGNATURE_SIZE];
        slice.copy_from_slice(value);

        Ok(Self::Ed25519(Ed25519Signature::new(slice)))
    }

    /// Parse a Secp256k1 signature from raw bytes
    #[cfg(feature = "secp256k1")]
    #[cfg_attr(docsrs, doc(cfg(feature = "secp256k1")))]
    fn from_raw_secp256k1(value: &[u8]) -> Result<Self, Error> {
        let sig = Secp256k1Signature::try_from(value)
            .map_err(|_| Error::signature_invalid("malformed Secp256k1 signature".to_string()))?;

        Ok(Self::Secp256k1(sig))
    }

    /// Return the algorithm used to create this particular signature
    pub fn algorithm(&self) -> Algorithm {
        match self {
            Signature::Ed25519(_) => Algorithm::Ed25519,
            #[cfg(feature = "secp256k1")]
            Signature::Secp256k1(_) => Algorithm::EcdsaSecp256k1,
            Signature::None => Algorithm::Ed25519, /* It doesn't matter what algorithm an empty
                                                    * signature has. */
        }
    }

    /// Get Ed25519 signature
    pub fn ed25519(self) -> Option<Ed25519Signature> {
        match self {
            Signature::Ed25519(sig) => Some(sig),
            _ => None,
        }
    }

    /// Get Secp256k1 signature
    #[cfg(feature = "secp256k1")]
    #[cfg_attr(docsrs, doc(cfg(feature = "secp256k1")))]
    pub fn secp256k1(self) -> Option<Secp256k1Signature> {
        match self {
            Signature::Secp256k1(sig) => Some(sig),
            _ => None,
        }
    }

    /// Return the raw bytes of this signature
    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    /// Get a vector containing the byte serialization of this key
    pub fn to_bytes(self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        match self {
            Signature::Ed25519(sig) => sig.as_ref(),
            #[cfg(feature = "secp256k1")]
            Signature::Secp256k1(sig) => sig.as_ref(),
            Signature::None => &[],
        }
    }
}

impl From<Ed25519Signature> for Signature {
    fn from(pk: Ed25519Signature) -> Signature {
        Signature::Ed25519(pk)
    }
}

#[cfg(feature = "secp256k1")]
impl From<Secp256k1Signature> for Signature {
    fn from(pk: Secp256k1Signature) -> Signature {
        Signature::Secp256k1(pk)
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

#[cfg(test)]
mod tests {

    #[test]
    #[cfg(feature = "secp256k1")]
    fn parse_secp256k1() {
        use super::*;
        use rand::SeedableRng;

        let rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
        let signing_key = k256::ecdsa::SigningKey::random(rng);
        let sig: Secp256k1Signature = signing_key.sign(&[1, 2, 3, 4, 5]);
        let sig_bytes = sig.as_ref();
        let sig = Signature::from_raw_secp256k1(sig_bytes).unwrap();
        dbg!(&sig);
        assert!(matches!(sig, Signature::Secp256k1(_)));
    }
}
