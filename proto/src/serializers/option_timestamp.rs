//! Serialize/deserialize Option<Timestamp> type from and into string:
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

use crate::google::protobuf::Timestamp;
use chrono::{DateTime, LocalResult, SecondsFormat, TimeZone, Utc};
use serde::ser::Error;

/// Deserialize string into Timestamp
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Timestamp>, D::Error>
where
    D: Deserializer<'de>,
{
    let value_string = String::deserialize(deserializer)?;
    let value_datetime = DateTime::parse_from_rfc3339(value_string.as_str())
        .map_err(|e| D::Error::custom(format!("{}", e)))?;
    Ok(Some(Timestamp {
        seconds: value_datetime.timestamp(),
        nanos: value_datetime.timestamp_subsec_nanos() as i32,
    }))
}

/// Serialize from Timestamp into string
pub fn serialize<S>(value: &Option<Timestamp>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value = value.as_ref().ok_or(S::Error::custom("no time found"))?;
    if value.nanos < 0 {
        return Err(S::Error::custom("invalid nanoseconds in time"));
    }
    match Utc.timestamp_opt(value.seconds, value.nanos as u32) {
        LocalResult::None => Err(S::Error::custom("invalid time")),
        LocalResult::Single(t) => Ok(t.to_rfc3339_opts(SecondsFormat::Millis, true)),
        LocalResult::Ambiguous(_, _) => Err(S::Error::custom("ambiguous time")),
    }?
    .serialize(serializer)
}
