//! Timestamps used by Tendermint blockchains

use crate::error::{Error, Kind};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use std::convert::{Infallible, TryFrom};
use std::fmt;
use std::ops::{Add, Sub};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tendermint_proto::google::protobuf::Timestamp;
use tendermint_proto::serializers::timestamp;
use tendermint_proto::Protobuf;

/// Tendermint timestamps
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#time>
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "Timestamp", into = "Timestamp")]
pub struct Time(DateTime<Utc>);

impl Protobuf<Timestamp> for Time {}

impl TryFrom<Timestamp> for Time {
    type Error = Infallible;

    fn try_from(value: Timestamp) -> Result<Self, Self::Error> {
        // prost_types::Timestamp has a SystemTime converter but
        // tendermint_proto::Timestamp can be JSON-encoded
        let prost_value = prost_types::Timestamp {
            seconds: value.seconds,
            nanos: value.nanos,
        };

        Ok(SystemTime::from(prost_value).into())
    }
}

impl From<Time> for Timestamp {
    fn from(value: Time) -> Self {
        // prost_types::Timestamp has a SystemTime converter but
        // tendermint_proto::Timestamp can be JSON-encoded
        let prost_value = prost_types::Timestamp::from(SystemTime::from(value));
        Timestamp {
            seconds: prost_value.seconds,
            nanos: prost_value.nanos,
        }
    }
}

impl Time {
    /// Get [`Time`] value representing the current wall clock time
    pub fn now() -> Self {
        Time(Utc::now())
    }

    /// Get the [`UNIX_EPOCH`] time ("1970-01-01 00:00:00 UTC") as a [`Time`]
    pub fn unix_epoch() -> Self {
        UNIX_EPOCH.into()
    }

    /// Calculate the amount of time which has passed since another [`Time`]
    /// as a [`std::time::Duration`]
    pub fn duration_since(&self, other: Time) -> Result<Duration, Error> {
        self.0
            .signed_duration_since(other.0)
            .to_std()
            .map_err(|_| Kind::OutOfRange.into())
    }

    /// Parse [`Time`] from an RFC 3339 date
    pub fn parse_from_rfc3339(s: &str) -> Result<Time, Error> {
        Ok(Time(DateTime::parse_from_rfc3339(s)?.with_timezone(&Utc)))
    }

    /// Return an RFC 3339 and ISO 8601 date and time string with 6 subseconds digits and Z.
    pub fn to_rfc3339(&self) -> String {
        timestamp::to_rfc3339_nanos(&self.0)
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

impl From<DateTime<Utc>> for Time {
    fn from(t: DateTime<Utc>) -> Time {
        Time(t)
    }
}

impl From<Time> for DateTime<Utc> {
    fn from(t: Time) -> DateTime<Utc> {
        t.0
    }
}

impl From<SystemTime> for Time {
    fn from(t: SystemTime) -> Time {
        Time(t.into())
    }
}

impl From<Time> for SystemTime {
    fn from(t: Time) -> SystemTime {
        t.0.into()
    }
}

impl Add<Duration> for Time {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        let st: SystemTime = self.into();
        (st + rhs).into()
    }
}

impl Sub<Duration> for Time {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        let st: SystemTime = self.into();
        (st - rhs).into()
    }
}

/// Parse [`Time`] from a type
pub trait ParseTimestamp {
    /// Parse [`Time`], or return an [`Error`] if parsing failed
    fn parse_timestamp(&self) -> Result<Time, Error>;
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;

    // TODO(shon) Extract arbitrary generators into own library

    use chrono::{DateTime, NaiveDate, TimeZone, Timelike, Utc};
    use proptest::{prelude::*, sample::select};

    // Any higher, and we're at seconds
    const MAX_NANO_SECS: u32 = 999_999_999u32;

    // With values larger or smaller then these, chrono produces invalid rfc3339
    // timestamps. See https://github.com/chronotope/chrono/issues/537
    fn min_time() -> DateTime<Utc> {
        Utc.timestamp(-9999999999, 0)
    }

    fn max_time() -> DateTime<Utc> {
        Utc.timestamp(99999999999, 0)
    }

    fn num_days_in_month(year: i32, month: u32) -> u32 {
        // Using chrono, we get the duration beteween this month and the next,
        // then count the number of days in that duration.  See
        // https://stackoverflow.com/a/58188385/1187277
        let given_month = NaiveDate::from_ymd(year, month, 1);
        let next_month = NaiveDate::from_ymd(
            if month == 12 { year + 1 } else { year },
            if month == 12 { 1 } else { month + 1 },
            1,
        );
        next_month
            .signed_duration_since(given_month)
            .num_days()
            .try_into()
            .unwrap()
    }

    prop_compose! {
        /// An abitrary `chrono::DateTime` that is between `min` and `max`
        /// DateTimes.
        fn arb_datetime_in_range(min: DateTime<Utc>, max: DateTime<Utc>)(
            secs in min.timestamp()..max.timestamp()
        )(
            // min mano secods is only relevant if we happen to hit the minimum
            // seconds on the nose.
            nano in (if secs == min.timestamp() { min.nanosecond() } else { 0 })..MAX_NANO_SECS,
            // Make secs in scope
            secs in Just(secs),
        ) -> DateTime<Utc> {
            println!(">> Secs {:?}", secs);
            Utc.timestamp(secs, nano)
        }
    }

    prop_compose! {
        /// An abitrary `chrono::DateTime`
        fn arb_datetime()
            (
                d in arb_datetime_in_range(min_time(), max_time())
            ) -> DateTime<Utc> {
                d
            }
    }

    prop_compose! {
        fn arb_rfc339_time_offset()(
            sign in "[+-]",
            hour in 0..23u8,
            min in 0..59u8,
        ) -> String {
            format!("{:}{:0>2}:{:0>2}", sign, hour, min)
        }
    }

    fn arb_rfc3339_offset() -> impl Strategy<Value = String> {
        prop_oneof![arb_rfc339_time_offset(), Just("Z".to_owned())]
    }

    prop_compose! {
        fn arb_rfc3339_partial_time()(
            hour in 0..23u8,
            min in 0..59u8,
            sec in 0..59u8,
            secfrac in proptest::option::of(0..u64::MAX),
        ) -> String {
            let frac = match secfrac {
                None => "".to_owned(),
                Some(frac) => format!(".{:}", frac)
            };
            format!("{:0>2}:{:0>2}:{:0>2}{:}", hour, min, sec, frac)
        }
    }

    prop_compose! {
        fn arb_rfc3339_full_time()(
            time in arb_rfc3339_partial_time(),
            offset in arb_rfc3339_offset()
        ) -> String {
            format!("{:}{:}", time, offset)
        }
    }

    prop_compose! {
        fn arb_rfc3339_day_of_year_and_month(year: i32, month: u32)
            (
                d in 1..num_days_in_month(year, month)
            ) -> u32 {
            d
        }
    }

    prop_compose! {
        fn arb_rfc3339_full_date()(year in 0..9999i32, month in 1..12u32)
            (
                day in arb_rfc3339_day_of_year_and_month(year, month),
                year in Just(year),
                month in Just(month),
            ) -> String {
                format!("{:0>4}-{:0>2}-{:0>2}", year, month, day)
            }
    }

    prop_compose! {
        /// An aribtrary rfc3339 timestamp
        ///
        /// Follows https://tools.ietf.org/html/rfc3339#section-5.6
        fn arb_rfc3339_timestamp()(
            date in arb_rfc3339_full_date(),
            time in arb_rfc3339_full_time()
        ) -> String {
            format!("{:}T{:}", date, time)
        }
    }

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
        fn can_parse_rfc3339_timestamps(stamp in arb_rfc3339_timestamp()) {
            prop_assert!(stamp.parse::<Time>().is_ok())
        }

        #[test]
        fn serde_from_value_is_the_inverse_of_to_value_within_reasonable_time_range(
            datetime in arb_datetime()
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
                arb_rfc3339_timestamp(),
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
