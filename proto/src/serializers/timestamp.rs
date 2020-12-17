//! Serialize/deserialize Timestamp type from and into string:
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

use crate::google::protobuf::Timestamp;
use chrono::{DateTime, Datelike, LocalResult, TimeZone, Timelike, Utc};
use serde::ser::Error;

/// Helper struct to serialize and deserialize Timestamp into an RFC3339-compatible string
/// This is required because the serde `with` attribute is only available to fields of a struct but
/// not the whole struct.
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct Rfc3339(#[serde(with = "crate::serializers::timestamp")] Timestamp);
impl From<Timestamp> for Rfc3339 {
    fn from(value: Timestamp) -> Self {
        Rfc3339(value)
    }
}
impl From<Rfc3339> for Timestamp {
    fn from(value: Rfc3339) -> Self {
        value.0
    }
}

/// Deserialize string into Timestamp
pub fn deserialize<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
where
    D: Deserializer<'de>,
{
    let value_string = String::deserialize(deserializer)?;
    let value_datetime = DateTime::parse_from_rfc3339(value_string.as_str())
        .map_err(|e| D::Error::custom(format!("{}", e)))?;
    Ok(Timestamp {
        seconds: value_datetime.timestamp(),
        nanos: value_datetime.timestamp_subsec_nanos() as i32,
    })
}

/// Serialize from Timestamp into string
pub fn serialize<S>(value: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if value.nanos < 0 {
        return Err(S::Error::custom("invalid nanoseconds in time"));
    }
    match Utc.timestamp_opt(value.seconds, value.nanos as u32) {
        LocalResult::None => Err(S::Error::custom("invalid time")),
        LocalResult::Single(t) => Ok(to_rfc3339_custom(&t)),
        LocalResult::Ambiguous(_, _) => Err(S::Error::custom("ambiguous time")),
    }?
    .serialize(serializer)
}

/// Serialization helper for converting a `DateTime<Utc>` object to a string.
///
/// Due to incompatibilities between the way that `chrono` serializes timestamps
/// and the way that Go does for RFC3339, we unfortunately need to define our
/// own timestamp serialization mechanism.
pub fn to_rfc3339_custom(t: &DateTime<Utc>) -> String {
    let nanos = format!(".{}", t.nanosecond());
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}Z",
        t.year(),
        t.month(),
        t.day(),
        t.hour(),
        t.minute(),
        t.second(),
        nanos.trim_end_matches('0').trim_end_matches('.'),
    )
}

#[cfg(test)]
mod test {
    use crate::google::protobuf::Timestamp;
    use serde::{Deserialize, Serialize};

    // The Go code with which the following timestamps were tested is as
    // follows:
    //
    // ```go
    // package main
    //
    // import (
    // 	"fmt"
    // 	"time"
    // )
    //
    // func main() {
    // 	timestamps := []string{
    // 		"2020-09-14T16:33:54.21191421Z",
    // 		"2020-09-14T16:33:00Z",
    // 		"2020-09-14T16:33:00.1Z",
    // 		"2020-09-14T16:33:00.211914212Z",
    // 	}
    // 	for _, timestamp := range timestamps {
    // 		ts, err := time.Parse(time.RFC3339Nano, timestamp)
    // 		if err != nil {
    // 			panic(err)
    // 		}
    // 		tss := ts.Format(time.RFC3339Nano)
    // 		if timestamp != tss {
    // 			panic(fmt.Sprintf("\nExpected : %s\nActual   : %s", timestamp, tss))
    // 		}
    // 	}
    // 	fmt.Println("All good!")
    // }
    // ```
    #[test]
    fn json_timestamp_precision() {
        #[derive(Serialize, Deserialize)]
        struct TimestampWrapper {
            timestamp: Timestamp,
        }
        let test_timestamps = vec![
            "2020-09-14T16:33:54.21191421Z",
            "2020-09-14T16:33:00Z",
            "2020-09-14T16:33:00.1Z",
            "2020-09-14T16:33:00.211914212Z",
            "1970-01-01T00:00:00Z",
            "0001-01-01T00:00:00Z",
        ];
        for timestamp in test_timestamps {
            let json = "{\"timestamp\":\"".to_owned() + timestamp + "\"}";
            let wrapper = serde_json::from_str::<TimestampWrapper>(&json).unwrap();
            assert_eq!(json, serde_json::to_string(&wrapper).unwrap());
        }
    }
}
