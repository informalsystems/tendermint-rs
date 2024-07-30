// Original code from <https://github.com/influxdata/pbjson/blob/main/pbjson-types/src/duration.rs>
// Copyright 2022 Dan Burkert & Tokio Contributors
//
// Original serialization code from <https://github.com/influxdata/pbjson/blob/main/pbjson-types/src/duration.rs>
// Copyright (c) 2020 InfluxData

use prost::Name;

use crate::prelude::*;

use super::type_url::type_url_for;
use super::PACKAGE;

/// A Duration represents a signed, fixed-length span of time represented
/// as a count of seconds and fractions of seconds at nanosecond
/// resolution. It is independent of any calendar and concepts like "day"
/// or "month". It is related to Timestamp in that the difference between
/// two Timestamp values is a Duration and it can be added or subtracted
/// from a Timestamp. Range is approximately +-10,000 years.
#[derive(Copy, Clone, PartialEq, ::prost::Message)]
pub struct Duration {
    /// Signed seconds of the span of time. Must be from -315,576,000,000
    /// to +315,576,000,000 inclusive. Note: these bounds are computed from:
    /// 60 sec/min * 60 min/hr * 24 hr/day * 365.25 days/year * 10000 years
    #[prost(int64, tag = "1")]
    pub seconds: i64,
    /// Signed fractions of a second at nanosecond resolution of the span
    /// of time. Durations less than one second are represented with a 0
    /// `seconds` field and a positive or negative `nanos` field. For durations
    /// of one second or more, a non-zero value for the `nanos` field must be
    /// of the same sign as the `seconds` field. Must be from -999,999,999
    /// to +999,999,999 inclusive.
    #[prost(int32, tag = "2")]
    pub nanos: i32,
}

impl Name for Duration {
    const PACKAGE: &'static str = PACKAGE;
    const NAME: &'static str = "Duration";

    fn type_url() -> String {
        type_url_for::<Self>()
    }
}

impl TryFrom<Duration> for core::time::Duration {
    type Error = core::num::TryFromIntError;

    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        Ok(Self::new(
            value.seconds.try_into()?,
            value.nanos.try_into()?,
        ))
    }
}

impl From<core::time::Duration> for Duration {
    fn from(value: core::time::Duration) -> Self {
        Self {
            seconds: value.as_secs() as _,
            nanos: value.subsec_nanos() as _,
        }
    }
}

impl serde::Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.seconds != 0 && self.nanos != 0 && (self.nanos < 0) != (self.seconds < 0) {
            return Err(serde::ser::Error::custom("Duration has inconsistent signs"));
        }

        let mut s = if self.seconds == 0 {
            if self.nanos < 0 {
                "-0".to_string()
            } else {
                "0".to_string()
            }
        } else {
            self.seconds.to_string()
        };

        if self.nanos != 0 {
            s.push('.');
            let f = match split_nanos(self.nanos.unsigned_abs()) {
                (millis, 0, 0) => format!("{:03}", millis),
                (millis, micros, 0) => format!("{:03}{:03}", millis, micros),
                (millis, micros, nanos) => format!("{:03}{:03}{:03}", millis, micros, nanos),
            };
            s.push_str(&f);
        }

        s.push('s');
        serializer.serialize_str(&s)
    }
}

struct DurationVisitor;

impl<'de> serde::de::Visitor<'de> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("a duration string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let s = s
            .strip_suffix('s')
            .ok_or_else(|| serde::de::Error::custom("missing 's' suffix"))?;

        let (negative, s) = match s.strip_prefix('-') {
            Some(s) => (true, s),
            None => (false, s),
        };

        let duration = match s.split_once('.') {
            Some((seconds_str, decimal_str)) => {
                let exp = 9_u32
                    .checked_sub(decimal_str.len() as u32)
                    .ok_or_else(|| serde::de::Error::custom("too many decimal places"))?;

                let pow = 10_u32.pow(exp);
                let seconds = seconds_str.parse().map_err(serde::de::Error::custom)?;
                let decimal: u32 = decimal_str.parse().map_err(serde::de::Error::custom)?;

                Duration {
                    seconds,
                    nanos: (decimal * pow) as i32,
                }
            },
            None => Duration {
                seconds: s.parse().map_err(serde::de::Error::custom)?,
                nanos: 0,
            },
        };

        Ok(match negative {
            true => Duration {
                seconds: -duration.seconds,
                nanos: -duration.nanos,
            },
            false => duration,
        })
    }
}

impl<'de> serde::Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(DurationVisitor)
    }
}

/// Splits nanoseconds into whole milliseconds, microseconds, and nanoseconds
fn split_nanos(mut nanos: u32) -> (u32, u32, u32) {
    let millis = nanos / 1_000_000;
    nanos -= millis * 1_000_000;
    let micros = nanos / 1_000;
    nanos -= micros * 1_000;
    (millis, micros, nanos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration() {
        let verify = |duration: &Duration, expected: &str| {
            assert_eq!(serde_json::to_string(duration).unwrap().as_str(), expected);
            assert_eq!(
                &serde_json::from_str::<Duration>(expected).unwrap(),
                duration
            )
        };

        let duration = Duration {
            seconds: 0,
            nanos: 0,
        };
        verify(&duration, "\"0s\"");

        let duration = Duration {
            seconds: 0,
            nanos: 123,
        };
        verify(&duration, "\"0.000000123s\"");

        let duration = Duration {
            seconds: 0,
            nanos: 123456,
        };
        verify(&duration, "\"0.000123456s\"");

        let duration = Duration {
            seconds: 0,
            nanos: 123456789,
        };
        verify(&duration, "\"0.123456789s\"");

        let duration = Duration {
            seconds: 0,
            nanos: -67088,
        };
        verify(&duration, "\"-0.000067088s\"");

        let duration = Duration {
            seconds: 121,
            nanos: 3454,
        };
        verify(&duration, "\"121.000003454s\"");

        let duration = Duration {
            seconds: -90,
            nanos: -2456301,
        };
        verify(&duration, "\"-90.002456301s\"");

        let duration = Duration {
            seconds: -90,
            nanos: 234,
        };
        serde_json::to_string(&duration).unwrap_err();

        let duration = Duration {
            seconds: 90,
            nanos: -234,
        };
        serde_json::to_string(&duration).unwrap_err();

        serde_json::from_str::<Duration>("90.1234567891s").unwrap_err();
    }
}
