//! AppHash serialization with validation

use core::str::FromStr;

use alloc::borrow::Cow;

use serde::{de, Deserialize, Deserializer, Serializer};

use crate::{prelude::*, AppHash};

/// Deserialize hexstring into AppHash
pub fn deserialize<'de, D>(deserializer: D) -> Result<AppHash, D::Error>
where
    D: Deserializer<'de>,
{
    let hexstring = Option::<Cow<'_, str>>::deserialize(deserializer)?.unwrap_or(Cow::Borrowed(""));
    AppHash::from_str(&hexstring).map_err(de::Error::custom)
}

/// Serialize from AppHash into hexstring
pub fn serialize<S>(value: &AppHash, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Serialize as Option<String> for symmetry with deserialize
    serializer.serialize_some(&value.to_string())
}
