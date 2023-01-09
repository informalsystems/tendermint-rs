//! Encoding/decoding Tendermint hashes to/from base64.

use serde::{Deserialize, Deserializer, Serializer};
use subtle_encoding::base64;
use tendermint::hash::{Algorithm::Sha256, Hash, SHA256_HASH_SIZE};

use crate::prelude::*;

/// Deserialize a base64-encoded string into an tendermint::Hash
pub fn deserialize<'de, D>(deserializer: D) -> Result<Hash, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?.unwrap_or_default();
    let decoded = base64::decode(s).map_err(serde::de::Error::custom)?;
    if decoded.len() != SHA256_HASH_SIZE {
        return Err(serde::de::Error::custom(
            "unexpected transaction length for hash",
        ));
    }
    let mut decoded_bytes = [0u8; SHA256_HASH_SIZE];
    decoded_bytes.copy_from_slice(decoded.as_ref());
    Hash::from_bytes(Sha256, &decoded_bytes).map_err(serde::de::Error::custom)
}

/// Serialize from a tendermint::Hash into a base64-encoded string
pub fn serialize<S>(value: &Hash, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let base64_bytes = base64::encode(value.as_bytes());
    let base64_string = String::from_utf8(base64_bytes).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&base64_string)
}
