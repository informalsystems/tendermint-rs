//! Cryptographic private keys

pub use ed25519_consensus::SigningKey as Ed25519;

use crate::prelude::*;
use crate::public_key::PublicKey;
use ed25519_consensus::VerificationKey;
use serde::{de, ser, Deserialize, Serialize};
use subtle_encoding::{Base64, Encoding};
use zeroize::Zeroizing;

pub const ED25519_KEYPAIR_SIZE: usize = 64;

/// Private keys as parsed from configuration files
#[derive(Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", content = "value")] // JSON custom serialization for priv_validator_key.json
pub enum PrivateKey {
    /// Ed25519 keys
    #[serde(
        rename = "tendermint/PrivKeyEd25519",
        serialize_with = "serialize_ed25519_keypair",
        deserialize_with = "deserialize_ed25519_keypair"
    )]
    Ed25519(Ed25519),
}

impl PrivateKey {
    /// Get the public key associated with this private key
    pub fn public_key(&self) -> PublicKey {
        match self {
            PrivateKey::Ed25519(signing_key) => PublicKey::Ed25519(signing_key.verification_key()),
        }
    }

    /// If applicable, borrow the Ed25519 keypair
    pub fn ed25519_signing_key(&self) -> Option<&Ed25519> {
        match self {
            PrivateKey::Ed25519(signing_key) => Some(signing_key),
        }
    }
}

/// Serialize an Ed25519 keypair as Base64
fn serialize_ed25519_keypair<S>(signing_key: &Ed25519, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    // Tendermint uses a serialization format inherited from Go that includes
    // a cached copy of the public key as the second half.
    let mut keypair_bytes = Zeroizing::new([0u8; ED25519_KEYPAIR_SIZE]);
    keypair_bytes[0..32].copy_from_slice(signing_key.as_bytes());
    keypair_bytes[32..64].copy_from_slice(signing_key.verification_key().as_bytes());
    Zeroizing::new(String::from_utf8(Base64::default().encode(&keypair_bytes[..])).unwrap())
        .serialize(serializer)
}

/// Deserialize an Ed25519 keypair from Base64
fn deserialize_ed25519_keypair<'de, D>(deserializer: D) -> Result<Ed25519, D::Error>
where
    D: de::Deserializer<'de>,
{
    use de::Error;
    let string = Zeroizing::new(String::deserialize(deserializer)?);
    let mut keypair_bytes = Zeroizing::new([0u8; ED25519_KEYPAIR_SIZE]);
    let decoded_len = Base64::default()
        .decode_to_slice(string.as_bytes(), &mut *keypair_bytes)
        .map_err(D::Error::custom)?;

    if decoded_len != ED25519_KEYPAIR_SIZE {
        return Err(D::Error::custom("invalid Ed25519 keypair size"));
    }

    // Tendermint uses a serialization format inherited from Go that includes a
    // cached copy of the public key as the second half.  This is somewhat
    // dangerous, since there's no validation that the two parts are consistent
    // with each other, so we ignore the second half and just check consistency
    // with the re-derived data.
    let signing_key = Ed25519::try_from(&keypair_bytes[0..32])
        .map_err(|_| D::Error::custom("invalid signing key"))?;
    let verification_key = VerificationKey::from(&signing_key);
    if &keypair_bytes[32..64] != verification_key.as_bytes() {
        return Err(D::Error::custom("keypair mismatch"));
    }

    Ok(signing_key)
}
