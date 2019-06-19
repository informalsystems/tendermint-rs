//! Cryptographic (a.k.a. digital) signatures

#[cfg(feature = "serde")]
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "serde")]
use signatory::Signature as SignatureTrait;
#[cfg(feature = "serde")]
use subtle_encoding::base64;

/// Signatures
#[derive(Clone, Debug)]
pub enum Signature {
    /// Ed25519 block signature
    Ed25519(signatory::ed25519::Signature),
}

impl Signature {
    /// Return the algorithm used to create this particular signature
    pub fn algorithm(self) -> Algorithm {
        match self {
            Signature::Ed25519(_) => Algorithm::Ed25519,
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = base64::decode(String::deserialize(deserializer)?.as_bytes())
            .map_err(|e| D::Error::custom(format!("{}", e)))?;

        Ok(Signature::Ed25519(
            signatory::ed25519::Signature::from_bytes(&bytes)
                .map_err(|e| D::Error::custom(format!("{}", e)))?,
        ))
    }
}

#[cfg(feature = "serde")]
impl Serialize for Signature {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let sig_bytes = match self {
            Signature::Ed25519(sig) => sig.as_bytes(),
        };

        String::from_utf8(base64::encode(&sig_bytes[..]))
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
