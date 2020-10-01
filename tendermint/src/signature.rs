//! Cryptographic (a.k.a. digital) signatures

pub use ed25519::{Signature as Ed25519Signature, SIGNATURE_LENGTH as ED25519_SIGNATURE_SIZE};
use signature::Signature as SignatureTrait;
pub use signature::{Signer, Verifier};

#[cfg(feature = "secp256k1")]
pub use k256::ecdsa::Signature as Secp256k1;

use crate::chain;
use crate::consensus::State;
use crate::{Error, Kind};
use bytes::BufMut;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use subtle_encoding::base64;
use tendermint_proto::DomainType;
use tendermint_proto::Error as DomainTypeError;

/// Signatures
#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Signature {
    /// Ed25519 block signature
    Ed25519(Ed25519Signature),
}

impl DomainType<Vec<u8>> for Signature {}

impl TryFrom<Vec<u8>> for Signature {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() != ED25519_SIGNATURE_SIZE {
            return Err(Kind::InvalidSignatureIDLength.into());
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

impl Signature {
    /// Return the algorithm used to create this particular signature
    pub fn algorithm(&self) -> Algorithm {
        match self {
            Signature::Ed25519(_) => Algorithm::Ed25519,
        }
    }

    /// Get Ed25519 signature
    pub fn ed25519(self) -> Option<Ed25519Signature> {
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

impl From<Ed25519Signature> for Signature {
    fn from(pk: Ed25519Signature) -> Signature {
        Signature::Ed25519(pk)
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = base64::decode(String::deserialize(deserializer)?.as_bytes())
            .map_err(D::Error::custom)?;

        Ed25519Signature::from_bytes(&bytes)
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

/// Messages which are signable within a Tendermint network
pub trait SignableMsg {
    /// Sign this message as bytes
    fn sign_bytes<B: BufMut>(&self, chain_id: chain::Id, sign_bytes: &mut B)
        -> Result<bool, Error>;
    /// Sign this message and return Vec<u8>
    fn sign_vec(&self, chain_id: chain::Id) -> Result<Vec<u8>, DomainTypeError>;
    /// Set the Ed25519 signature on the underlying message
    fn set_signature(&mut self, sig: Signature);
    /// Get consensus state // Todo: Do we need this? It used to be implemented for Amino.
    fn consensus_state(&self) -> Option<State>;
}
