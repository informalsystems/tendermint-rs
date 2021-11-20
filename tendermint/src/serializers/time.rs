//! An alternative timestamp serialization/deserialization mechanism for
//! RFC3339-compatible timestamps to that provided by the `tendermint-proto`
//! crate.

use crate::prelude::*;
use crate::Time;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serialize from `Time` into `String`
pub fn serialize<S>(value: &Time, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    value.to_rfc3339().serialize(serializer)
}

/// Deserialize `String` into `Time`
pub fn deserialize<'de, D>(deserializer: D) -> Result<Time, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Time::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)
}
