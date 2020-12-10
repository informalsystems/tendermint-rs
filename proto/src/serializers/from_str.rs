//! Serialize and deserialize any `T` that implements [[std::str::FromStr]]
//! and [[std::fmt::Display]] from or into string. Note this can be used for
//! all primitive data types.
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

/// Deserialize string into T
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    String::deserialize(deserializer)?
        .parse::<T>()
        .map_err(|e| D::Error::custom(format!("{}", e)))
}

/// Serialize from T into string
pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: std::fmt::Display,
{
    format!("{}", value).serialize(serializer)
}
