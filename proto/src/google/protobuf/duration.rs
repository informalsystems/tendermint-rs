// Original code from <https://github.com/influxdata/pbjson/blob/main/pbjson-types/src/duration.rs>
// Copyright 2022 Dan Burkert & Tokio Contributors
//
// Original serialization code from <https://github.com/influxdata/pbjson/blob/main/pbjson-types/src/duration.rs>
// Copyright (c) 2020 InfluxData

use core::convert::TryFrom;

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
#[derive(Copy, Clone, PartialEq, Eq, ::prost::Message)]
#[cfg_attr(feature = "json-schema", derive(::schemars::JsonSchema))]
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

const NANOS_PER_SECOND: i32 = 1_000_000_000;
const NANOS_MAX: i32 = NANOS_PER_SECOND - 1;

impl Duration {
    /// Normalizes the duration to a canonical format.
    pub fn normalize(&mut self) {
        // Make sure nanos is in the range.
        if self.nanos <= -NANOS_PER_SECOND || self.nanos >= NANOS_PER_SECOND {
            if let Some(seconds) = self
                .seconds
                .checked_add((self.nanos / NANOS_PER_SECOND) as i64)
            {
                self.seconds = seconds;
                self.nanos %= NANOS_PER_SECOND;
            } else if self.nanos < 0 {
                // Negative overflow! Set to the least normal value.
                self.seconds = i64::MIN;
                self.nanos = -NANOS_MAX;
            } else {
                // Positive overflow! Set to the greatest normal value.
                self.seconds = i64::MAX;
                self.nanos = NANOS_MAX;
            }
        }

        // nanos should have the same sign as seconds.
        if self.seconds < 0 && self.nanos > 0 {
            if let Some(seconds) = self.seconds.checked_add(1) {
                self.seconds = seconds;
                self.nanos -= NANOS_PER_SECOND;
            } else {
                // Positive overflow! Set to the greatest normal value.
                debug_assert_eq!(self.seconds, i64::MAX);
                self.nanos = NANOS_MAX;
            }
        } else if self.seconds > 0 && self.nanos < 0 {
            if let Some(seconds) = self.seconds.checked_sub(1) {
                self.seconds = seconds;
                self.nanos += NANOS_PER_SECOND;
            } else {
                // Negative overflow! Set to the least normal value.
                debug_assert_eq!(self.seconds, i64::MIN);
                self.nanos = -NANOS_MAX;
            }
        }
    }
}

/// Converts a `core::time::Duration` to a `Duration`.
impl From<core::time::Duration> for Duration {
    fn from(duration: core::time::Duration) -> Duration {
        let seconds = duration.as_secs();
        let seconds = if seconds > i64::MAX as u64 {
            i64::MAX
        } else {
            seconds as i64
        };
        let nanos = duration.subsec_nanos();
        let nanos = if nanos > i32::MAX as u32 {
            i32::MAX
        } else {
            nanos as i32
        };
        let mut duration = Duration { seconds, nanos };
        duration.normalize();
        duration
    }
}

impl TryFrom<Duration> for core::time::Duration {
    type Error = core::time::Duration;

    /// Converts a `Duration` to a result containing a positive (`Ok`) or negative (`Err`)
    /// `std::time::Duration`.
    fn try_from(mut duration: Duration) -> Result<core::time::Duration, core::time::Duration> {
        duration.normalize();
        if duration.seconds >= 0 {
            Ok(core::time::Duration::new(
                duration.seconds as u64,
                duration.nanos as u32,
            ))
        } else {
            Err(core::time::Duration::new(
                (-duration.seconds) as u64,
                (-duration.nanos) as u32,
            ))
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
