//! `Option<Hash>` serialization with validation

use alloc::borrow::Cow;

use serde::{de, Deserialize, Deserializer, Serializer};

use super::hash;
use crate::{hash::Algorithm, Hash};

/// Deserialize a nullable hexstring into `Option<Hash>`.
/// A null value is deserialized as `None`.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Hash>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<Cow<'_, str>>::deserialize(deserializer)? {
        Some(s) => Hash::from_hex_upper(Algorithm::Sha256, &s)
            .map(Some)
            .map_err(de::Error::custom),
        None => Ok(None),
    }
}

/// Serialize from `Option<Hash>` into a nullable hexstring. None is serialized as null.
pub fn serialize<S>(value: &Option<Hash>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if value.is_none() {
        serializer.serialize_none()
    } else {
        // hash::serialize serializes as Option<String>, so this is consistent
        // with the other branch.
        hash::serialize(value.as_ref().unwrap(), serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_round_trip() {
        let v: Option<Hash> = None;
        let json = serde_json::to_string(&v).unwrap();
        let parsed: Option<Hash> = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_none());
    }
}
