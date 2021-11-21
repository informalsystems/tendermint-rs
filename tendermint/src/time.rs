//! Timestamps used by Tendermint blockchains

use serde::{Deserialize, Serialize};

use crate::prelude::*;
use core::convert::{TryFrom, TryInto};
use core::fmt;
use core::ops::{Add, Sub};
use core::str::FromStr;
use core::time::Duration;
use tendermint_proto::google::protobuf::Timestamp;
use tendermint_proto::serializers::timestamp;
use tendermint_proto::Protobuf;
use time::format_description::well_known::Rfc3339;
use time::macros::{datetime, offset};
use time::{OffsetDateTime, PrimitiveDateTime};

use crate::error::Error;

/// Tendermint timestamps
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#time>
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "Timestamp", into = "Timestamp")]
pub struct Time(PrimitiveDateTime);

impl Protobuf<Timestamp> for Time {}

impl TryFrom<Timestamp> for Time {
    type Error = Error;

    fn try_from(value: Timestamp) -> Result<Self, Error> {
        let nanos = value.nanos.try_into().map_err(Error::timestamp_overflow)?;
        Time::from_unix_timestamp(value.seconds, nanos)
    }
}

impl From<Time> for Timestamp {
    fn from(value: Time) -> Self {
        let total_nanos = value.0.assume_utc().unix_timestamp_nanos();
        Timestamp {
            seconds: total_nanos.div_euclid(1_000_000_000) as _,
            nanos: total_nanos.rem_euclid(1_000_000_000) as _,
        }
    }
}

impl Time {
    fn from_utc(odt: OffsetDateTime) -> Self {
        assert_eq!(odt.offset(), offset!(UTC));
        Time(PrimitiveDateTime::new(odt.date(), odt.time()))
    }

    /// Get the unix epoch ("1970-01-01 00:00:00 UTC") as a [`Time`]
    pub fn unix_epoch() -> Self {
        Time(datetime!(1970-01-01 00:00:00))
    }

    pub fn from_unix_timestamp(secs: i64, nanos: u32) -> Result<Self, Error> {
        let total_nanos = secs as i128 * 1_000_000_000 + nanos as i128;
        match OffsetDateTime::from_unix_timestamp_nanos(total_nanos) {
            Ok(odt) => Ok(Self::from_utc(odt)),
            _ => Err(Error::timestamp_conversion()),
        }
    }

    /// Calculate the amount of time which has passed since another [`Time`]
    /// as a [`core::time::Duration`]
    pub fn duration_since(&self, other: Time) -> Result<Duration, Error> {
        let duration = self.0.assume_utc() - other.0.assume_utc();
        let duration = duration
            .try_into()
            .map_err(|_| Error::duration_out_of_range())?;
        Ok(duration)
    }

    /// Parse [`Time`] from an RFC 3339 date
    pub fn parse_from_rfc3339(s: &str) -> Result<Time, Error> {
        let date = OffsetDateTime::parse(s, &Rfc3339)
            .map_err(Error::time_parse)?
            .to_offset(offset!(UTC));
        Ok(Time::from_utc(date))
    }

    /// Return an RFC 3339 and ISO 8601 date and time string with subseconds (if nonzero) and Z.
    pub fn to_rfc3339(&self) -> String {
        timestamp::to_rfc3339_nanos(self.0.assume_utc())
    }

    /// Computes `self + duration`, returning `None` if an overflow occurred.
    pub fn checked_add(self, duration: Duration) -> Option<Self> {
        let duration = duration.try_into().ok()?;
        self.0.checked_add(duration).map(Time)
    }

    /// Computes `self - duration`, returning `None` if an overflow occurred.
    pub fn checked_sub(self, duration: Duration) -> Option<Self> {
        let duration = duration.try_into().ok()?;
        self.0.checked_sub(duration).map(Time)
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.to_rfc3339())
    }
}

impl FromStr for Time {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Time::parse_from_rfc3339(s)
    }
}

impl From<OffsetDateTime> for Time {
    fn from(t: OffsetDateTime) -> Time {
        Time::from_utc(t.to_offset(offset!(UTC)))
    }
}

impl From<Time> for OffsetDateTime {
    fn from(t: Time) -> OffsetDateTime {
        t.0.assume_utc()
    }
}

impl Add<Duration> for Time {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self {
        Time(self.0 + rhs)
    }
}

impl Sub<Duration> for Time {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        Time(self.0 - rhs)
    }
}

/// Parse [`Time`] from a type
pub trait ParseTimestamp {
    /// Parse [`Time`], or return an [`Error`] if parsing failed
    fn parse_timestamp(&self) -> Result<Time, Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{prelude::*, sample::select};
    use tendermint_pbt_gen as pbt;

    // We want to make sure that these timestamps specifically get tested.
    fn particular_rfc3339_timestamps() -> impl Strategy<Value = String> {
        let strs: Vec<String> = vec![
            "2020-09-14T16:33:54.21191421Z",
            "2020-09-14T16:33:00Z",
            "2020-09-14T16:33:00.1Z",
            "2020-09-14T16:33:00.211914212Z",
            "1970-01-01T00:00:00Z",
            "2021-01-07T20:25:56.0455760Z",
            "2021-01-07T20:25:57.039219Z",
            "2021-01-07T20:25:58.03562100Z",
            "2021-01-07T20:25:59.000955200Z",
            "2021-01-07T20:26:04.0121030Z",
            "2021-01-07T20:26:05.005096Z",
            "2021-01-07T20:26:09.08488400Z",
            "2021-01-07T20:26:11.0875340Z",
            "2021-01-07T20:26:12.078268Z",
            "2021-01-07T20:26:13.08074100Z",
            "2021-01-07T20:26:15.079663000Z",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        select(strs)
    }

    proptest! {
        #[test]
        fn can_parse_rfc3339_timestamps(stamp in pbt::time::arb_rfc3339_timestamp()) {
            prop_assert!(stamp.parse::<Time>().is_ok())
        }

        #[test]
        fn serde_from_value_is_the_inverse_of_to_value_within_reasonable_time_range(
            datetime in pbt::time::arb_datetime()
        ) {
            // If `from_value` is the inverse of `to_value`, then it will always
            // map the JSON `encoded_time` to back to the inital `time`.
            let time: Time = datetime.into();
            let json_encoded_time = serde_json::to_value(&time).unwrap();
            let decoded_time: Time = serde_json::from_value(json_encoded_time).unwrap();
            prop_assert_eq!(time, decoded_time);
        }

        #[test]
        fn serde_of_rfc3339_timestamps_is_safe(
            stamp in prop_oneof![
                pbt::time::arb_rfc3339_timestamp(),
                particular_rfc3339_timestamps(),
            ]
        ) {
            // ser/de of rfc3339 timestamps is safe if it never panics.
            // This differes from the the inverse test in that we are testing on
            // arbitrarily generated textual timestamps, rather than times in a
            // range. Tho we do incidentally test the inversion as well.
            let time: Time = stamp.parse().unwrap();
            let json_encoded_time = serde_json::to_value(&time).unwrap();
            let decoded_time: Time = serde_json::from_value(json_encoded_time).unwrap();
            prop_assert_eq!(time, decoded_time);
        }
    }
}
