//! Serialize/deserialize Option<T> type where T has a serializer/deserializer.
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Deserialize Option<T>
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Some(T::deserialize(deserializer)?))
}

/// Serialize Option<T>
pub fn serialize<S, T>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Clone + Default + Serialize,
{
    value.clone().unwrap_or_default().serialize(serializer)
}
