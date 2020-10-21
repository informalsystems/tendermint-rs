//! Serialize/deserialize Option<Timestamp> type from and into string:
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

use crate::google::protobuf::Timestamp;
use chrono::{DateTime, Datelike, LocalResult, TimeZone, Timelike, Utc};
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
    let value = value
        .as_ref()
        .ok_or_else(|| S::Error::custom("no time found"))?;
    if value.nanos < 0 {
        return Err(S::Error::custom("invalid nanoseconds in time"));
    }
    match Utc.timestamp_opt(value.seconds, value.nanos as u32) {
        LocalResult::None => Err(S::Error::custom("invalid time")),
        LocalResult::Single(t) => Ok(to_rfc3999(t)),
        LocalResult::Ambiguous(_, _) => Err(S::Error::custom("ambiguous time")),
    }?
    .serialize(serializer)
}

// Due to incompatibilities between the way that `chrono` serializes timestamps
// and the way that Go does for RFC3339, we unfortunately need to define our
// own timestamp serialization mechanism.
fn to_rfc3999(t: DateTime<Utc>) -> String {
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
            #[serde(with = "crate::serializers::option_timestamp")]
            timestamp: Option<Timestamp>,
        }
        let test_timestamps = vec![
            "2020-09-14T16:33:54.21191421Z",
            "2020-09-14T16:33:00Z",
            "2020-09-14T16:33:00.1Z",
            "2020-09-14T16:33:00.211914212Z",
        ];
        for timestamp in test_timestamps {
            let json = "{\"timestamp\":\"".to_owned() + timestamp + "\"}";
            let wrapper = serde_json::from_str::<TimestampWrapper>(&json).unwrap();
            assert_eq!(json, serde_json::to_string(&wrapper).unwrap());
        }
    }
}
