//! Option<Hash> serialization with validation

use super::hash;
use crate::Hash;
use serde::{Deserializer, Serializer};

/// Deserialize hexstring into Option<Hash>
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Hash>, D::Error>
where
    D: Deserializer<'de>,
{
    hash::deserialize(deserializer).map(Some)
}

/// Serialize from Option<Hash> into hexstring
pub fn serialize<S>(value: &Option<Hash>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if value.is_none() {
        serializer.serialize_none()
    } else {
        hash::serialize(&value.unwrap(), serializer)
    }
}
