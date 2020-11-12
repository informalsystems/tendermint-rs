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
    if incoming_string.is_empty() {
        return Ok(serde_json::Value::Null);
    }
    // Try to deserialize as-is (usually sequence/array) or as string with extra quotes.
    serde_json::Value::from_str(&incoming_string).or_else(|e| {
        serde_json::Value::from_str(&format!("\"{}\"", incoming_string))
            .map_err(|_| D::Error::custom(format!("{}", e)))
    })
}

/// Serialize from serde_json::Value into string
pub fn serialize<S>(value: &serde_json::Value, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Remove extra quotes that might have been added during deserialization.
    if value.is_string() {
        let v = value.to_string();
        let v = v.strip_prefix(r#"""#).unwrap_or(&v);
        v.strip_suffix(r#"""#).unwrap_or(&v).serialize(serializer)
    } else {
        value.to_string().serialize(serializer)
    }
}

/// Helper struct to serialize/deserialize log messages
#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub struct Log(#[serde(with = "crate::serializers::log")] pub serde_json::Value);
