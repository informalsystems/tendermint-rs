//! Serialize/deserialize Timestamp type from and into string:

use crate::google::protobuf::Timestamp;
use crate::prelude::*;

use core::fmt;
use serde::de::Error as _;
use serde::ser::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use time::format_description::well_known::Rfc3339 as Rfc3339Format;
use time::macros::offset;
use time::OffsetDateTime;

/// Helper struct to serialize and deserialize Timestamp into an RFC3339-compatible string
/// This is required because the serde `with` attribute is only available to fields of a struct but
/// not the whole struct.
#[derive(Debug, Serialize, Deserialize)]
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
    let t = OffsetDateTime::parse(&value_string, &Rfc3339Format).map_err(D::Error::custom)?;
    let t = t.to_offset(offset!(UTC));
    if !matches!(t.year(), 1..=9999) {
        return Err(D::Error::custom("date is out of range"));
    }
    let seconds = t.unix_timestamp();
    // Safe to convert to i32 because .nanosecond()
    // is guaranteed to return a value in 0..1_000_000_000 range.
    let nanos = t.nanosecond() as i32;
    Ok(Timestamp { seconds, nanos })
}

/// Serialize from Timestamp into string
pub fn serialize<S>(value: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if value.nanos < 0 || value.nanos > 999_999_999 {
        return Err(S::Error::custom("invalid nanoseconds in time"));
    }
    let total_nanos = value.seconds as i128 * 1_000_000_000 + value.nanos as i128;
    let datetime = OffsetDateTime::from_unix_timestamp_nanos(total_nanos)
        .map_err(|_| S::Error::custom("invalid time"))?;
    to_rfc3339_nanos(datetime).serialize(serializer)
}

/// Serialization helper for converting an [`OffsetDateTime`] object to a string.
///
/// This reproduces the behavior of Go's `time.RFC3339Nano` format,
/// ie. a RFC3339 date-time with left-padded subsecond digits without
///     trailing zeros and no trailing dot.
pub fn to_rfc3339_nanos(t: OffsetDateTime) -> String {
    // Can't use OffsetDateTime::format because the feature enabling it
    // currently requires std (https://github.com/time-rs/time/issues/400)

    // Preallocate enough string capacity to fit the shortest possible form,
    // yyyy-mm-ddThh:mm:ssZ
    let mut buf = String::with_capacity(20);

    fmt_as_rfc3339_nanos(t, &mut buf).unwrap();

    buf
}

/// Helper for formatting an [`OffsetDateTime`] value.
///
/// This function can be used to efficiently format date-time values
/// in [`Display`] or [`Debug`] implementations.
///
/// The format reproduces Go's `time.RFC3339Nano` format,
/// ie. a RFC3339 date-time with left-padded subsecond digits without
///     trailing zeros and no trailing dot.
///
/// [`Display`]: core::fmt::Display
/// [`Debug`]: core::fmt::Debug
pub fn fmt_as_rfc3339_nanos(t: OffsetDateTime, f: &mut impl fmt::Write) -> fmt::Result {
    let t = t.to_offset(offset!(UTC));
    let nanos = t.nanosecond();
    if nanos == 0 {
        write!(
            f,
            "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z",
            year = t.year(),
            month = t.month() as u8,
            day = t.day(),
            hour = t.hour(),
            minute = t.minute(),
            second = t.second(),
        )
    } else {
        let mut secfrac = nanos;
        let mut secfrac_width = 9;
        while secfrac % 10 == 0 {
            secfrac /= 10;
            secfrac_width -= 1;
        }
        write!(
            f,
            "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{secfrac:0sfw$}Z",
            year = t.year(),
            month = t.month() as u8,
            day = t.day(),
            hour = t.hour(),
            minute = t.minute(),
            second = t.second(),
            secfrac = secfrac,
            sfw = secfrac_width,
        )
    }
}

#[allow(warnings)]
#[cfg(test)]
mod test {
    use super::*;
    use crate::google::protobuf::Timestamp;
    use serde::{Deserialize, Serialize};

    // The Go code with which the following timestamps
    // were tested is as follows:
    //
    // ```go
    // package main
    //
    // import (
    //     "fmt"
    //     "time"
    // )
    //
    // func main() {
    //     timestamps := []string{
    //         "1970-01-01T00:00:00Z",
    //         "0001-01-01T00:00:00Z",
    //         "2020-09-14T16:33:00Z",
    //         "2020-09-14T16:33:00.1Z",
    //         "2020-09-14T16:33:00.211914212Z",
    //         "2020-09-14T16:33:54.21191421Z",
    //         "2021-01-07T20:25:56.045576Z",
    //         "2021-01-07T20:25:57.039219Z",
    //         "2021-01-07T20:26:05.00509Z",
    //         "2021-01-07T20:26:05.005096Z",
    //         "2021-01-07T20:26:05.0005096Z",
    //     }
    //     for _, timestamp := range timestamps {
    //         ts, err := time.Parse(time.RFC3339Nano, timestamp)
    //         if err != nil {
    //             panic(err)
    //         }
    //         tss := ts.Format(time.RFC3339Nano)
    //         if timestamp != tss {
    //             panic(fmt.Sprintf("\nExpected : %s\nActual   : %s", timestamp, tss))
    //         }
    //     }
    //     fmt.Println("All good!")
    // }
    // ```
    #[test]
    fn json_timestamp_precision() {
        let test_timestamps = vec![
            "1970-01-01T00:00:00Z",
            "0001-01-01T00:00:00Z",
            "2020-09-14T16:33:00Z",
            "2020-09-14T16:33:00.1Z",
            "2020-09-14T16:33:00.211914212Z",
            "2020-09-14T16:33:54.21191421Z",
            "2021-01-07T20:25:56.045576Z",
            "2021-01-07T20:25:57.039219Z",
            "2021-01-07T20:26:05.00509Z",
            "2021-01-07T20:26:05.005096Z",
            "2021-01-07T20:26:05.0005096Z",
        ];

        for timestamp in test_timestamps {
            let json = format!("\"{}\"", timestamp);
            let rfc = serde_json::from_str::<Rfc3339>(&json).unwrap();
            assert_eq!(json, serde_json::to_string(&rfc).unwrap());
        }
    }
}
