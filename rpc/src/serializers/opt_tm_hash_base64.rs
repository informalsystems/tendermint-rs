//! Encoding/decoding Option Tendermint hashes to/from base64.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tendermint::hash::Hash;

use crate::prelude::*;

#[derive(Serialize, Deserialize)]
struct Helper(#[serde(with = "crate::serializers::tm_hash_base64")] Hash);

/// Deserialize base64-encoded string into an Option<tendermint::Hash>
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Hash>, D::Error>
where
    D: Deserializer<'de>,
{
    let helper: Option<Helper> = Option::deserialize(deserializer)?;
    Ok(helper.map(|Helper(hash)| hash))
}

/// Serialize from an Option<tendermint::Hash> into a base64-encoded string
pub fn serialize<S>(value: &Option<Hash>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    value.map(Helper).serialize(serializer)
}
