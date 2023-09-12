//! AppHash serialization with validation

use serde::{de, Deserialize, Deserializer, Serializer};
use subtle_encoding::base64;

use crate::{prelude::*, AppHash};

/// Deserialize a base64-encoded string into an [`AppHash`]
pub fn deserialize<'de, D>(deserializer: D) -> Result<AppHash, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<&str>::deserialize(deserializer)?.unwrap_or("");
    let decoded = base64::decode(s).map_err(de::Error::custom)?;
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
