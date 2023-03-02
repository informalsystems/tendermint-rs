//! Serialize/deserialize `nil`able value into `T`, where `nil` turns into the `Default` value.
//!
//! Serialize any value of `T`, including the default value,
//! using the serialization for `Some` variant of `Option<T>`.
//!
//! This helper can be used to tolerate `nil` values from a serialization producer,
//! while the default value is normatively serialized as such.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Deserialize `T` from a `nil`-able representation, accepting the `nil`
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

/// Serialize `T` as `Some` value of `Option<T>`.
pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    serializer.serialize_some(value)
}
