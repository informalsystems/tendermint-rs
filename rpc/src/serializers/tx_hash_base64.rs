//! Encoding/decoding ABCI transaction hashes to/from base64.

use serde::{Deserialize, Deserializer, Serializer};
use subtle_encoding::base64;

use crate::prelude::*;
use tendermint::{hash::Algorithm, Hash};

/// Deserialize a base64-encoded string into an abci::transaction::Hash
pub fn deserialize<'de, D>(deserializer: D) -> Result<Hash, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?.unwrap_or_default();
    let decoded = base64::decode(s).map_err(serde::de::Error::custom)?;
    let hash = Hash::from_bytes(Algorithm::Sha256, &decoded).map_err(serde::de::Error::custom)?;
    Ok(hash)
}

/// Serialize from an abci::transaction::Hash into a base64-encoded string
pub fn serialize<S>(value: &Hash, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let base64_bytes = base64::encode(value.as_bytes());
    let base64_string = String::from_utf8(base64_bytes).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&base64_string)
}
