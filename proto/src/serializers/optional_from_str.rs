//! De/serialize an optional type that must be converted from/to a string.

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serializer};
use std::str::FromStr;

pub fn serialize<S, T>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToString,
{
    match value {
        Some(t) => serializer.serialize_some(&t.to_string()),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: std::error::Error,
{
    let s = match Option::<String>::deserialize(deserializer)? {
        Some(s) => s,
        None => return Ok(None),
    };
    Ok(Some(s.parse().map_err(|e: <T as FromStr>::Err| {
        D::Error::custom(e.to_string())
    })?))
}
