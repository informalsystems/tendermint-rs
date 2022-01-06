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
///
/// A `Time` value is guaranteed to represent a valid `Timestamp` as defined
/// by Google's well-known protobuf type [specification]. Conversions and
/// operations that would result in exceeding `Timestamp`'s validity
/// range return an error or `None`.
///
/// The string serialization format for `Time` is defined as an RFC 3339
/// compliant string with the optional subsecond fraction part having
/// up to 9 digits and no trailing zeros, and the UTC offset denoted by Z.
/// This reproduces the behavior of Go's `time.RFC3339Nano` format.
///
/// [specification]: https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.Timestamp
// For memory efficiency, the inner member is `PrimitiveDateTime`, with assumed
// UTC offset. The `assume_utc` method is used to get the operational
// `OffsetDateTime` value.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "Timestamp", into = "Timestamp")]
pub struct Time(PrimitiveDateTime);

impl Protobuf<Timestamp> for Time {}

impl TryFrom<Timestamp> for Time {
    type Error = Error;

    fn try_from(value: Timestamp) -> Result<Self, Error> {
        let nanos = value
            .nanos
            .try_into()
            .map_err(|_| Error::timestamp_nanos_out_of_range())?;
        Self::from_unix_timestamp(value.seconds, nanos)
    }
}

impl From<Time> for Timestamp {
    fn from(value: Time) -> Self {
        let t = value.0.assume_utc();
        let seconds = t.unix_timestamp();
        // Safe to convert to i32 because .nanosecond()
        // is guaranteed to return a value in 0..1_000_000_000 range.
        let nanos = t.nanosecond() as i32;
        Timestamp { seconds, nanos }
    }
}

impl Time {
    #[cfg(any(feature = "clock"))]
    pub fn now() -> Time {
        OffsetDateTime::now_utc().try_into().unwrap()
    }

    // Internal helper to produce a `Time` value validated with regard to
    // the date range allowed in protobuf timestamps.
    // The source `OffsetDateTime` value must have the zero UTC offset.
    fn from_utc(t: OffsetDateTime) -> Result<Self, Error> {
        debug_assert_eq!(t.offset(), offset!(UTC));
        match t.year() {
            1..=9999 => Ok(Self(PrimitiveDateTime::new(t.date(), t.time()))),
            _ => Err(Error::date_out_of_range()),
        }
    }

    /// Get the unix epoch ("1970-01-01 00:00:00 UTC") as a [`Time`]
    pub fn unix_epoch() -> Self {
        Self(datetime!(1970-01-01 00:00:00))
    }

    pub fn from_unix_timestamp(secs: i64, nanos: u32) -> Result<Self, Error> {
        if nanos > 999_999_999 {
            return Err(Error::timestamp_nanos_out_of_range());
        }
        let total_nanos = secs as i128 * 1_000_000_000 + nanos as i128;
        match OffsetDateTime::from_unix_timestamp_nanos(total_nanos) {
            Ok(odt) => Self::from_utc(odt),
            _ => Err(Error::timestamp_conversion()),
        }
    }

    /// Calculate the amount of time which has passed since another [`Time`]
    /// as a [`core::time::Duration`]
    pub fn duration_since(&self, other: Time) -> Result<Duration, Error> {
        let duration = self.0.assume_utc() - other.0.assume_utc();
        duration
            .try_into()
            .map_err(|_| Error::duration_out_of_range())
    }

    /// Parse [`Time`] from an RFC 3339 date
    pub fn parse_from_rfc3339(s: &str) -> Result<Self, Error> {
        let date = OffsetDateTime::parse(s, &Rfc3339)
            .map_err(Error::time_parse)?
            .to_offset(offset!(UTC));
        Self::from_utc(date)
    }

    /// Return an RFC 3339 and ISO 8601 date and time string with subseconds (if nonzero) and Z.
    pub fn to_rfc3339(&self) -> String {
        timestamp::to_rfc3339_nanos(self.0.assume_utc())
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        timestamp::fmt_as_rfc3339_nanos(self.0.assume_utc(), f)
    }
}

impl FromStr for Time {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_from_rfc3339(s)
    }
}

impl TryFrom<OffsetDateTime> for Time {
    type Error = Error;

    fn try_from(t: OffsetDateTime) -> Result<Time, Error> {
        Self::from_utc(t.to_offset(offset!(UTC)))
    }
}

impl From<Time> for OffsetDateTime {
    fn from(t: Time) -> OffsetDateTime {
        t.0.assume_utc()
    }
}

impl Add<Duration> for Time {
    type Output = Result<Self, Error>;

    fn add(self, rhs: Duration) -> Self::Output {
        // Work around not being able to depend on time 0.3.5
        // https://github.com/informalsystems/tendermint-rs/issues/1047
        let lhs_nanos = self.0.assume_utc().unix_timestamp_nanos();
        let rhs_nanos: i128 = rhs
            .as_nanos()
            .try_into()
            .map_err(|_| Error::duration_out_of_range())?;
        let res_nanos = lhs_nanos
            .checked_add(rhs_nanos)
            .ok_or_else(Error::duration_out_of_range)?;
        let t = OffsetDateTime::from_unix_timestamp_nanos(res_nanos)
            .map_err(|_| Error::duration_out_of_range())?;
        Self::from_utc(t)
    }
}

impl Sub<Duration> for Time {
    type Output = Result<Self, Error>;

    fn sub(self, rhs: Duration) -> Self::Output {
        // Work around not being able to depend on time 0.3.5
        // https://github.com/informalsystems/tendermint-rs/issues/1047
        let lhs_nanos = self.0.assume_utc().unix_timestamp_nanos();
        let rhs_nanos: i128 = rhs
            .as_nanos()
            .try_into()
            .map_err(|_| Error::duration_out_of_range())?;
        let res_nanos = lhs_nanos
            .checked_sub(rhs_nanos)
            .ok_or_else(Error::duration_out_of_range)?;
        let t = OffsetDateTime::from_unix_timestamp_nanos(res_nanos)
            .map_err(|_| Error::duration_out_of_range())?;
        Self::from_utc(t)
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
    use crate::error::ErrorDetail;
    use proptest::{prelude::*, sample::select};
    use tendermint_pbt_gen as pbt;
    use time::{Date, Month::*};

    // We want to make sure that these timestamps specifically get tested.
    fn particular_rfc3339_timestamps() -> impl Strategy<Value = String> {
        let strs: Vec<String> = vec![
            "0001-01-01T00:00:00Z",
            "9999-12-31T23:59:59.999999999Z",
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

    fn particular_datetimes_out_of_range() -> impl Strategy<Value = OffsetDateTime> {
        let dts = vec![
            datetime!(0000-12-31 23:59:59.999999999 UTC),
            datetime!(0001-01-01 00:00:00.999999999 +00:00:01),
            datetime!(9999-12-31 23:59:59 -00:00:01),
            Date::from_calendar_date(-1, October, 9)
                .unwrap()
                .midnight()
                .assume_utc(),
        ];
        select(dts)
    }

    proptest! {
        #[test]
        fn can_parse_rfc3339_timestamps(stamp in pbt::time::arb_protobuf_safe_rfc3339_timestamp()) {
            prop_assert!(stamp.parse::<Time>().is_ok())
        }

        #[test]
        fn serde_from_value_is_the_inverse_of_to_value_within_reasonable_time_range(
            datetime in pbt::time::arb_protobuf_safe_datetime()
        ) {
            // If `from_value` is the inverse of `to_value`, then it will always
            // map the JSON `encoded_time` to back to the inital `time`.
            let time: Time = datetime.try_into().unwrap();
            let json_encoded_time = serde_json::to_value(&time).unwrap();
            let decoded_time: Time = serde_json::from_value(json_encoded_time).unwrap();
            prop_assert_eq!(time, decoded_time);
        }

        #[test]
        fn serde_of_rfc3339_timestamps_is_safe(
            stamp in prop_oneof![
                pbt::time::arb_protobuf_safe_rfc3339_timestamp(),
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

        #[test]
        fn conversion_from_datetime_succeeds_for_4_digit_ce_years(
            datetime in prop_oneof![
                pbt::time::arb_datetime_with_offset(),
                particular_datetimes_out_of_range(),
            ]
        ) {
            let res: Result<Time, _> = datetime.try_into();
            match datetime.to_offset(offset!(UTC)).year() {
                1 ..= 9999 => {
                    let t = res.unwrap();
                    let dt_converted_back: OffsetDateTime = t.into();
                    assert_eq!(dt_converted_back, datetime);
                }
                _ => {
                    let e = res.unwrap_err();
                    assert!(matches!(e.detail(), ErrorDetail::DateOutOfRange(_)))
                }
            }
        }

        #[test]
        fn from_unix_timestamp_rejects_out_of_range_nanos(
            datetime in pbt::time::arb_protobuf_safe_datetime(),
            nanos in 1_000_000_000 ..= u32::MAX,
        ) {
            let secs = datetime.unix_timestamp();
            let res = Time::from_unix_timestamp(secs, nanos);
            let e = res.unwrap_err();
            assert!(matches!(e.detail(), ErrorDetail::TimestampNanosOutOfRange(_)))
        }
    }

    fn duration_from_nanos(whole_nanos: u128) -> Duration {
        let secs: u64 = (whole_nanos / 1_000_000_000).try_into().unwrap();
        let nanos = (whole_nanos % 1_000_000_000) as u32;
        Duration::new(secs, nanos)
    }

    prop_compose! {
        fn args_for_regular_add()
            (t in pbt::time::arb_protobuf_safe_datetime())
            (
                t in Just(t),
                d_nanos in 0 ..= (pbt::time::max_protobuf_time() - t).whole_nanoseconds() as u128,
            ) -> (OffsetDateTime, Duration)
            {
                (t, duration_from_nanos(d_nanos))
            }
    }

    prop_compose! {
        fn args_for_regular_sub()
            (t in pbt::time::arb_protobuf_safe_datetime())
            (
                t in Just(t),
                d_nanos in 0 ..= (t - pbt::time::min_protobuf_time()).whole_nanoseconds() as u128,
            ) -> (OffsetDateTime, Duration)
            {
                (t, duration_from_nanos(d_nanos))
            }
    }

    prop_compose! {
        fn args_for_overflowed_add()
            (t in pbt::time::arb_protobuf_safe_datetime())
            (
                t in Just(t),
                d_nanos in (
                    (pbt::time::max_protobuf_time() - t).whole_nanoseconds() as u128 + 1
                    ..=
                    Duration::MAX.as_nanos()
                ),
            ) -> (OffsetDateTime, Duration)
            {
                (t, duration_from_nanos(d_nanos))
            }
    }

    prop_compose! {
        fn args_for_overflowed_sub()
            (t in pbt::time::arb_protobuf_safe_datetime())
            (
                t in Just(t),
                d_nanos in (
                    (t - pbt::time::min_protobuf_time()).whole_nanoseconds() as u128 + 1
                    ..=
                    Duration::MAX.as_nanos()
                ),
            ) -> (OffsetDateTime, Duration)
            {
                (t, duration_from_nanos(d_nanos))
            }
    }

    proptest! {
        #[test]
        fn add_regular((dt, d) in args_for_regular_add()) {
            let t: Time = dt.try_into().unwrap();
            let t = (t + d).unwrap();
            let res: OffsetDateTime = t.into();
            assert_eq!(res, dt + d);
        }

        #[test]
        fn sub_regular((dt, d) in args_for_regular_sub()) {
            let t: Time = dt.try_into().unwrap();
            let t = (t - d).unwrap();
            let res: OffsetDateTime = t.into();
            assert_eq!(res, dt - d);
        }

        #[test]
        fn add_overflow((dt, d) in args_for_overflowed_add()) {
            let t: Time = dt.try_into().unwrap();
            assert!((t + d).is_err());
        }

        #[test]
        fn sub_overflow((dt, d) in args_for_overflowed_sub()) {
            let t: Time = dt.try_into().unwrap();
            assert!((t - d).is_err());
        }
    }
}
