//! Serialize/deserialize core::time::Duration type from and into string:
use core::time::Duration;

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

use crate::prelude::*;
use crate::serializers::cow_str::CowStr;

/// Deserialize string into Duration
pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let value = CowStr::deserialize(deserializer)?
        .parse::<u64>()
        .map_err(|e| D::Error::custom(format!("{e}")))?;

    Ok(Duration::from_nanos(value))
}

/// Serialize from Duration into string
pub fn serialize<S>(value: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    format!("{}", value.as_nanos()).serialize(serializer)
}
