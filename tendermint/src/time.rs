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

    use chrono::{DateTime, NaiveDate, TimeZone, Timelike, Utc};
    use proptest::prelude::*;

    fn year_zero() -> Time {
        Utc.yo(0, 1).and_hms_nano(0, 0, 0, 0).into()
    }

    fn max_time() -> Time {
        // Later times cannot be handled by chrono
        Utc.timestamp(7999999999999, 0).into()
    }

    // Via chrono, we get the duration beteween this month and the next, then
    // count the number of days in that duration.
    // See https://stackoverflow.com/a/58188385/1187277
    fn num_days_in_month(year: i32, month: u32) -> u32 {
        let given_month = NaiveDate::from_ymd(year, month, 1);
        let next_month = NaiveDate::from_ymd(
            if let 12 = month { year + 1 } else { year },
            if let 12 = month { 1 } else { month + 1 },
            1,
        );
        next_month
            .signed_duration_since(given_month)
            .num_days()
            .try_into()
            .unwrap()
    }

    type DT = DateTime<Utc>;

    prop_compose! {
        /// An abitrary `chrono::DateTime` that is later than `after`
        fn arb_datetime(min: Time, max: Time)(
            // This is near the maximum allowed seconds for a timestamp (for no clear reason I've found documented)
            // but it is in year 255479, so we should be good for a while.
            secs in DT::from(min).timestamp()..DT::from(max).timestamp(),
            // This is the max allowed value for nanoseconds (for some reason).
            // https://github.com/chronotope/chrono/blob/3467172c31188006147585f6ed3727629d642fed/src/naive/time.rs#L385
            // FIXME Logic is not right here, becuase if we're given a start time of 0.1, then 1.0 should be valid, but
            //       but this will enforce >= 1.1.
            nano in DT::from(min).nanosecond()..999_999_999u32
        ) -> DateTime<Utc> {
            println!("Secs {:?}; Nano {:?}", secs, nano);
            Utc.timestamp(secs, nano)
        }
    }

    prop_compose! {
        /// An abitrary `Time` that is `betweeon` `min` and `max` `Time`s, if
        /// given, or between the `year_zero` and `max_time` otherwise.
        fn arb_time_in_range(min: Option<Time>, max: Option<Time>)
            (
                d in arb_datetime(min.unwrap_or(year_zero()), max.unwrap_or(max_time()))
            ) -> Time {
                Time::from(d)
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

    fn min_reasonable_time() -> Time {
        // Guarantee we'll be good for any time starting from when Delton 3030
        // was released.
        DateTime::parse_from_rfc3339("2000-05-23T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
            .into()
    }

    fn max_reasonable_time() -> Time {
        // Guarantee we'll be good for any time until Deltron Zero arrives.
        DateTime::parse_from_rfc3339("3030-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
            .into()
    }

    proptest! {
        #[test]
        fn serde_from_value_is_the_inverse_of_to_value_within_reasonable_time_range(
            time in arb_time_in_range(Some(min_reasonable_time()),
                                      Some(max_reasonable_time()))
        ) {
            // If `from_value` is the inverse of `to_value`, then it will always
            // map the JSON `encoded_time` to back to the inital `time`.
            let json_encoded_time = serde_json::to_value(&time).unwrap();
            let decoded_time: Time = serde_json::from_value(json_encoded_time.clone()).unwrap();
            prop_assert_eq!(time, decoded_time);
        }

        #[test]
        fn can_parse_rfc_3339_timestamps(stamp in arb_rfc3339_timestamp()) {
            prop_assert!(stamp.parse::<Time>().is_ok())
        }

        #[test]
        fn serde_of_rfc_3339_time_stamps_is_safe(
            stamp in arb_rfc3339_timestamp()
        ) {
            // ser/de of rfc_3339 timestamps is safe if it never panics.
            let time = stamp.parse::<Time>().unwrap();
            let json_encoded_time = serde_json::to_value(&time).unwrap();
            let decoded_time: Time = serde_json::from_value(json_encoded_time.clone()).unwrap();
            prop_assert_eq!(time, decoded_time);
        }
    }

    #[test]
    fn serde_roundtrip() {
        const DATES: &[&str] = &[
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
        ];

        for input in DATES {
            let initial_time: Time = input.parse().unwrap();
            let encoded_time = serde_json::to_value(&initial_time).unwrap();
            let decoded_time = serde_json::from_value(encoded_time.clone()).unwrap();

            assert_eq!(initial_time, decoded_time);
        }
    }
}
