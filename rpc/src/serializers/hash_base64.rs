//! Encoding/decoding ABCI transaction hashes to/from base64.

use crate::abci::transaction::{Hash, HASH_LENGTH};
use crate::prelude::*;
use serde::{Deserialize, Deserializer, Serializer};
use subtle_encoding::base64;

/// Deserialize a base64-encoded string into a Hash
pub fn deserialize<'de, D>(deserializer: D) -> Result<Hash, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?.unwrap_or_default();
    let decoded = base64::decode(&s).map_err(serde::de::Error::custom)?;
    if decoded.len() != HASH_LENGTH {
        return Err(serde::de::Error::custom(
            "unexpected transaction length for hash",
        ));
    }
    let mut decoded_bytes = [0u8; HASH_LENGTH];
    decoded_bytes.copy_from_slice(decoded.as_ref());
    Ok(Hash::new(decoded_bytes))
}

/// Serialize from a Hash into a base64-encoded string
pub fn serialize<S>(value: &Hash, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let base64_bytes = base64::encode(value.as_bytes());
    let base64_string = String::from_utf8(base64_bytes).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&base64_string)
}
