//! Time-related data structures.

use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A simple wrapper structure for a `chrono::DateTime<Utc>` that facilitates
/// our serialization scheme for timestamps.
#[derive(Debug, Clone, PartialEq)]
pub struct Time(DateTime<Utc>);

impl From<DateTime<Utc>> for Time {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl From<Time> for DateTime<Utc> {
    fn from(t: Time) -> Self {
        t.0
    }
}

impl AsRef<DateTime<Utc>> for Time {
    fn as_ref(&self) -> &DateTime<Utc> {
        &self.0
    }
}

impl Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        to_rfc3339_custom(&self.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Time {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D>::Error>
    where
        D: Deserializer<'de>,
    {
        let value_string = String::deserialize(deserializer)?;
        let value_datetime = DateTime::parse_from_rfc3339(value_string.as_str())
            .map_err(|e| D::Error::custom(format!("{}", e)))?
            .with_timezone(&Utc);
        Ok(Time::from(value_datetime))
    }
}

// Due to incompatibilities between the way that `chrono` serializes timestamps
// and the way that Go does for RFC3339, we unfortunately need to define our
// own timestamp serialization mechanism.
fn to_rfc3339_custom(t: &DateTime<Utc>) -> String {
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
    use super::*;
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
        struct TimeWrapper {
            time: Time,
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
            let json = "{\"time\":\"".to_owned() + timestamp + "\"}";
            let wrapper = serde_json::from_str::<TimeWrapper>(&json).unwrap();
            assert_eq!(json, serde_json::to_string(&wrapper).unwrap());
        }
    }
}
