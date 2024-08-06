// Original code from <https://github.com/influxdata/pbjson/blob/main/pbjson-types/src/timestamp.rs>
// Copyright 2022 Dan Burkert & Tokio Contributors

use prost::Name;

use crate::prelude::*;

use super::type_url::type_url_for;
use super::PACKAGE;

/// A Timestamp represents a point in time independent of any time zone or local
/// calendar, encoded as a count of seconds and fractions of seconds at
/// nanosecond resolution. The count is relative to an epoch at UTC midnight on
/// January 1, 1970, in the proleptic Gregorian calendar which extends the
/// Gregorian calendar backwards to year one.
///
/// All minutes are 60 seconds long. Leap seconds are "smeared" so that no leap
/// second table is needed for interpretation, using a
/// [24-hour linear smear](https://developers.google.com/time/smear).
///
/// The range is from 0001-01-01T00:00:00Z to 9999-12-31T23:59:59.999999999Z. By
/// restricting to that range, we ensure that we can convert to and from
/// [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) date strings.
#[derive(Copy, Clone, PartialEq, ::prost::Message, ::serde::Deserialize, ::serde::Serialize)]
#[serde(
    from = "crate::serializers::timestamp::Rfc3339",
    into = "crate::serializers::timestamp::Rfc3339"
)]
#[cfg_attr(feature = "json-schema", derive(::schemars::JsonSchema))]
pub struct Timestamp {
    /// Represents seconds of UTC time since Unix epoch
    /// 1970-01-01T00:00:00Z. Must be from 0001-01-01T00:00:00Z to
    /// 9999-12-31T23:59:59Z inclusive.
    #[prost(int64, tag = "1")]
    pub seconds: i64,
    /// Non-negative fractions of a second at nanosecond resolution. Negative
    /// second values with fractions must still have non-negative nanos values
    /// that count forward in time. Must be from 0 to 999,999,999
    /// inclusive.
    #[prost(int32, tag = "2")]
    pub nanos: i32,
}

impl Name for Timestamp {
    const PACKAGE: &'static str = PACKAGE;
    const NAME: &'static str = "Timestamp";

    fn type_url() -> String {
        type_url_for::<Self>()
    }
}

const NANOS_PER_SECOND: i32 = 1_000_000_000;

impl Timestamp {
    /// Normalizes the timestamp to a canonical format.
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
                // Negative overflow! Set to the earliest normal value.
                self.seconds = i64::MIN;
                self.nanos = 0;
            } else {
                // Positive overflow! Set to the latest normal value.
                self.seconds = i64::MAX;
                self.nanos = 999_999_999;
            }
        }

        // For Timestamp nanos should be in the range [0, 999999999].
        if self.nanos < 0 {
            if let Some(seconds) = self.seconds.checked_sub(1) {
                self.seconds = seconds;
                self.nanos += NANOS_PER_SECOND;
            } else {
                // Negative overflow! Set to the earliest normal value.
                debug_assert_eq!(self.seconds, i64::MIN);
                self.nanos = 0;
            }
        }
    }
}

/// Implements the unstable/naive version of `Eq`: a basic equality check on the internal fields of the `Timestamp`.
/// This implies that `normalized_ts != non_normalized_ts` even if `normalized_ts == non_normalized_ts.normalized()`.
impl Eq for Timestamp {}

// Derived logic is correct: comparing the 2 fields for equality
#[allow(clippy::derived_hash_with_manual_eq)]
impl core::hash::Hash for Timestamp {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.seconds.hash(state);
        self.nanos.hash(state);
    }
}

#[cfg(feature = "std")]
impl From<std::time::SystemTime> for Timestamp {
    fn from(system_time: std::time::SystemTime) -> Timestamp {
        let (seconds, nanos) = match system_time.duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => {
                let seconds = i64::try_from(duration.as_secs()).unwrap();
                (seconds, duration.subsec_nanos() as i32)
            },
            Err(error) => {
                let duration = error.duration();
                let seconds = i64::try_from(duration.as_secs()).unwrap();
                let nanos = duration.subsec_nanos() as i32;
                if nanos == 0 {
                    (-seconds, 0)
                } else {
                    (-seconds - 1, 1_000_000_000 - nanos)
                }
            },
        };
        Timestamp { seconds, nanos }
    }
}

/// Indicates that a [`Timestamp`] could not be converted to
/// [`SystemTime`][std::time::SystemTime] because it is out of range.
///
/// The range of times that can be represented by `SystemTime` depends on the platform.
/// All `Timestamp`s are likely representable on 64-bit Unix-like platforms, but
/// other platforms, such as Windows and 32-bit Linux, may not be able to represent
/// the full range of `Timestamp`s.
#[cfg(feature = "std")]
#[derive(Debug)]
#[non_exhaustive]
pub struct TimestampOutOfSystemRangeError {
    pub timestamp: Timestamp,
}

#[cfg(feature = "std")]
impl core::fmt::Display for TimestampOutOfSystemRangeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{self:?} is not representable as a `SystemTime` because it is out of range"
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TimestampOutOfSystemRangeError {}

#[cfg(feature = "std")]
impl TryFrom<Timestamp> for std::time::SystemTime {
    type Error = TimestampOutOfSystemRangeError;

    fn try_from(mut timestamp: Timestamp) -> Result<std::time::SystemTime, Self::Error> {
        let orig_timestamp = timestamp;

        timestamp.normalize();

        let system_time = if timestamp.seconds >= 0 {
            std::time::UNIX_EPOCH
                .checked_add(core::time::Duration::from_secs(timestamp.seconds as u64))
        } else {
            std::time::UNIX_EPOCH
                .checked_sub(core::time::Duration::from_secs((-timestamp.seconds) as u64))
        };

        let system_time = system_time.and_then(|system_time| {
            system_time.checked_add(core::time::Duration::from_nanos(timestamp.nanos as u64))
        });

        system_time.ok_or(TimestampOutOfSystemRangeError {
            timestamp: orig_timestamp,
        })
    }
}
