//! Cryptographic private keys

pub use ed25519_dalek::{Keypair as Ed25519, EXPANDED_SECRET_KEY_LENGTH as ED25519_KEYPAIR_SIZE};

use crate::prelude::*;
use crate::public_key::PublicKey;
use serde::{de, ser, Deserialize, Serialize};
use subtle_encoding::{Base64, Encoding};
use zeroize::Zeroizing;

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
            PrivateKey::Ed25519(private_key) => private_key.public.into(),
        }
    }

    /// If applicable, borrow the Ed25519 keypair
    pub fn ed25519_keypair(&self) -> Option<&Ed25519> {
        match self {
            PrivateKey::Ed25519(keypair) => Some(keypair),
        }
    }
}

/// Serialize an Ed25519 keypair as Base64
fn serialize_ed25519_keypair<S>(keypair: &Ed25519, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    let keypair_bytes = Zeroizing::new(keypair.to_bytes());
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

    Ed25519::from_bytes(&*keypair_bytes).map_err(D::Error::custom)
}
