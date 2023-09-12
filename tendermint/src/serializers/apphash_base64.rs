//! AppHash serialization with validation

use alloc::borrow::Cow;

use serde::{de, Deserialize, Deserializer, Serializer};
use subtle_encoding::base64;

use crate::{prelude::*, AppHash};

/// Deserialize a base64-encoded string into an [`AppHash`]
pub fn deserialize<'de, D>(deserializer: D) -> Result<AppHash, D::Error>
where
    D: Deserializer<'de>,
{
    let decoded = match Option::<Cow<'_, str>>::deserialize(deserializer)? {
        Some(s) => base64::decode(s.as_bytes()).map_err(de::Error::custom)?,
        None => vec![],
    };
    decoded.try_into().map_err(de::Error::custom)
}

/// Serialize from [`AppHash`] into a base64-encoded string.
pub fn serialize<S>(value: &AppHash, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let base64_bytes = base64::encode(value.as_bytes());
    let base64_string = String::from_utf8(base64_bytes).unwrap();
    // Serialize as Option<String> for symmetry with deserialize
    serializer.serialize_some(&base64_string)
}
