//! AppHash serialization with validation

use alloc::borrow::Cow;

use serde::{de, ser, Deserialize, Deserializer, Serializer};
use subtle_encoding::hex;

use crate::{prelude::*, AppHash};

/// Deserialize hexstring into AppHash
pub fn deserialize<'de, D>(deserializer: D) -> Result<AppHash, D::Error>
where
    D: Deserializer<'de>,
{
    let hexstring = Option::<Cow<'_, str>>::deserialize(deserializer)?.unwrap_or(Cow::Borrowed(""));
    AppHash::from_hex_upper(&hexstring).map_err(de::Error::custom)
}

/// Serialize from AppHash into hexstring
pub fn serialize<S>(value: &AppHash, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let hex_bytes = hex::encode_upper(value.as_ref());
    let hex_string = String::from_utf8(hex_bytes).map_err(ser::Error::custom)?;
    // Serialize as Option<String> for symmetry with deserialize
    serializer.serialize_some(&hex_string)
}
