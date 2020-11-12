//! Serialize and deserialize serde_json::Value
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

/// Deserialize string into serde_json::Value
pub fn deserialize<'de, D>(deserializer: D) -> Result<serde_json::Value, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize into String first.
    // This has the added side-effect that it consumes escaped quotes in sub-structs,
    // deserializing them properly as Value.
    let incoming_string = String::deserialize(deserializer)?;
    serde_json::Value::from_str(&incoming_string).map_err(|e| D::Error::custom(format!("{}", e)))
}

/// Serialize from serde_json::Value into string
pub fn serialize<S>(value: &serde_json::Value, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Todo: Re-add quote escapes?
    value.to_string().serialize(serializer)
}

/// Helper struct to serialize/deserialize log messages
#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub struct Log(#[serde(with = "crate::serializers::log")] pub serde_json::Value);
