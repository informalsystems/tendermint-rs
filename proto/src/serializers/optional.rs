//! Serialize/deserialize Option<T> type where T has a serializer/deserializer.
//! Use `null` if the received value equals the Default implementation.
// Todo: Rename this serializer to something like "default_eq_none" to mirror its purpose better.
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Deserialize Option<T>
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default + PartialEq,
{
    Ok(Option::<T>::deserialize(deserializer)?.filter(|t| t != &T::default()))
}

/// Serialize Option<T>
pub fn serialize<S, T>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Clone + Default + Serialize,
{
    value.clone().unwrap_or_default().serialize(serializer)
}
