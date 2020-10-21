//! Custom, legacy serializers

use crate::Hash;
use serde::{de::Error as _, Deserialize, Deserializer};
use std::str::FromStr;

// Todo: Refactor the "Option"-based serializers below.
//  Most of them are not needed if the structs are defined well (with enums).

/// Option<Hash> deserialization
pub fn parse_non_empty_hash<'de, D>(deserializer: D) -> Result<Option<Hash>, D::Error>
where
    D: Deserializer<'de>,
{
    let o: Option<String> = Option::deserialize(deserializer)?;
    match o.filter(|s| !s.is_empty()) {
        None => Ok(None),
        Some(s) => Ok(Some(
            Hash::from_str(&s).map_err(|err| D::Error::custom(format!("{}", err)))?,
        )),
    }
}

/// Parse null as default
pub fn null_as_default<'de, D, T: Default + Deserialize<'de>>(
    deserializer: D,
) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(<Option<T>>::deserialize(deserializer)?.unwrap_or_default())
}
