//! Serialize and deserialize part_set_header.total (from string or u32), (into u32 in
//! part_set_header.total).
//!
//! The deserializer is created for backwards compatibility: `total` was changed from a
//! string-quoted integer value into an integer value without quotes in Tendermint Core v0.34.0.
//! This deserializer allows backwards-compatibility by deserializing both ways.
//! See also: <https://github.com/informalsystems/tendermint-rs/issues/679>
use serde::{de::Error, de::Visitor, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::fmt::Formatter;

struct PartSetHeaderTotalStringOrU32;

/// Deserialize (string or u32) into u32(part_set_header.total)
pub fn deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(PartSetHeaderTotalStringOrU32)
}

/// Serialize from u32(part_set_header.total) into u32
pub fn serialize<S>(value: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    value.serialize(serializer)
}

impl<'de> Visitor<'de> for PartSetHeaderTotalStringOrU32 {
    type Value = u32;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("an u32 integer or string between 0 and 2^32")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        u32::try_from(v).map_err(|e| E::custom(format!("part_set_header.total {}", e)))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        v.parse::<u32>()
            .map_err(|e| E::custom(format!("part_set_header.total {}", e)))
    }
}
