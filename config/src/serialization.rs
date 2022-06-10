//! Serialization utilities

use core::{fmt, str::FromStr};

use serde::{de, de::Error as _, ser, Deserialize, Serialize};

use crate::prelude::*;

/// Deserialize `Option<T: FromStr>` where an empty string indicates `None`
pub fn deserialize_optional_value<'de, D, T, E>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: de::Deserializer<'de>,
    T: FromStr<Err = E>,
    E: fmt::Display,
{
    let string = Option::<String>::deserialize(deserializer).map(|str| str.unwrap_or_default())?;

    if string.is_empty() {
        return Ok(None);
    }

    string
        .parse()
        .map(Some)
        .map_err(|e| D::Error::custom(format!("{}", e)))
}

pub fn serialize_optional_value<S, T>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
    T: Serialize,
{
    match value {
        Some(value) => value.serialize(serializer),
        None => "".serialize(serializer),
    }
}

/// Deserialize a comma separated list of types that impl `FromStr` as a `Vec`
pub fn deserialize_comma_separated_list<'de, D, T, E>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: de::Deserializer<'de>,
    T: FromStr<Err = E>,
    E: fmt::Display,
{
    let mut result = vec![];
    let string = String::deserialize(deserializer)?;

    if string.is_empty() {
        return Ok(result);
    }

    for item in string.split(',') {
        result.push(
            item.parse()
                .map_err(|e| D::Error::custom(format!("{}", e)))?,
        );
    }

    Ok(result)
}

/// Serialize a comma separated list types that impl `ToString`
pub fn serialize_comma_separated_list<S, T>(list: &[T], serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
    T: ToString,
{
    let str_list = list.iter().map(|addr| addr.to_string()).collect::<Vec<_>>();
    str_list.join(",").serialize(serializer)
}

/// Deserialize an item from a string that impl `ToString`.
pub fn deserialize_from_str<'de, D, T, E>(deserializer: D) -> Result<T, D::Error>
where
    D: de::Deserializer<'de>,
    T: FromStr<Err = E>,
    E: fmt::Display,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(|e| D::Error::custom(format!("{}", e)))
}

/// Serialize a value that impl `ToString` to a string.
pub fn serialize_to_str<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
    T: ToString,
{
    value.to_string().serialize(serializer)
}
