//! Provides [proptest](https://github.com/AltSysrq/proptest) generators for
//! time-like objects.

use std::convert::TryInto;

use proptest::prelude::*;
use time::format_description::well_known::Rfc3339;
use time::macros::{datetime, offset};
use time::{Date, OffsetDateTime, UtcOffset};

/// Any higher, and we're at seconds
pub const MAX_NANO_SECS: u32 = 999_999_999u32;

/// The most distant time in the past for which `time` produces correct
/// times with [`OffsetDateTime::from_unix_timestamp`].
///
/// ```
/// use tendermint_pbt_gen as pbt_gen;
/// use time::OffsetDateTime;
///
/// let timestamp = pbt_gen::time::min_time().unix_timestamp_nanos();
/// assert!(OffsetDateTime::from_unix_timestamp_nanos(timestamp).is_ok());
/// assert!(OffsetDateTime::from_unix_timestamp_nanos(timestamp - 1).is_err());
/// ```
pub fn min_time() -> OffsetDateTime {
    Date::MIN.midnight().assume_utc()
}

/// The most distant time in the future for which `time` produces correct
/// times with [`OffsetDateTime::from_unix_timestamp`].
///
/// ```
/// use tendermint_pbt_gen as pbt_gen;
/// use time::OffsetDateTime;
///
/// let timestamp = pbt_gen::time::max_time().unix_timestamp_nanos();
/// assert!(OffsetDateTime::from_unix_timestamp_nanos(timestamp).is_ok());
/// assert!(OffsetDateTime::from_unix_timestamp_nanos(timestamp + 1).is_err());
/// ```
pub fn max_time() -> OffsetDateTime {
    Date::MAX
        .with_hms_nano(23, 59, 59, MAX_NANO_SECS)
        .unwrap()
        .assume_utc()
}

/// The most distant time in the past that has a valid representation in
/// Google's well-known [`Timestamp`] protobuf message format.
///
/// [`Timestamp`]: https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.Timestamp
pub const fn min_protobuf_time() -> OffsetDateTime {
    datetime!(0001-01-01 00:00:00 UTC)
}

/// The most distant time in the future that has a valid representation in
/// Google's well-known [`Timestamp`] protobuf message format.
///
/// [`Timestamp`]: https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.Timestamp
pub const fn max_protobuf_time() -> OffsetDateTime {
    datetime!(9999-12-31 23:59:59.999999999 UTC)
}

fn num_days_in_month(year: i32, month: u8) -> u8 {
    let month = month.try_into().unwrap();
    time::util::days_in_year_month(year, month)
}

prop_compose! {
    /// An abitrary [`OffsetDateTime`], offset in UTC,
    /// that is between the given `min` and `max`.
    ///
    /// # Examples
    ///
    /// ```
    /// use time::macros::datetime;
    /// use tendermint_pbt_gen as pbt_gen;
    /// use proptest::prelude::*;
    ///
    /// proptest!{
    ///     fn rosa_luxemburg_and_octavia_butler_were_not_alive_at_the_same_time(
    ///        time_in_luxemburgs_lifespan in pbt_gen::time::arb_datetime_in_range(
    ///          datetime!(1871-03-05 00:00 UTC), // DOB
    ///          datetime!(1919-01-15 00:00 UTC), // DOD
    ///        ),
    ///        time_in_butlers_lifespan in pbt_gen::time::arb_datetime_in_range(
    ///          datetime!(1947-06-22 00:00 UTC), // DOB
    ///          datetime!(2006-02-24 00:00 UTC), // DOD
    ///        ),
    ///     ) {
    ///       prop_assert!(time_in_luxemburgs_lifespan != time_in_butlers_lifespan)
    ///     }
    /// }
    /// ```
    pub fn arb_datetime_in_range(min: OffsetDateTime, max: OffsetDateTime)(
        nanos in min.unix_timestamp_nanos()..max.unix_timestamp_nanos()
    ) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp_nanos(nanos).unwrap()
    }
}

prop_compose! {
    /// An abitrary [`OffsetDateTime`], offset in UTC (between [min_time] and [max_time]).
    pub fn arb_datetime()
        (
            d in arb_datetime_in_range(min_time(), max_time())
        ) -> OffsetDateTime {
            d
        }
}

prop_compose! {
    /// An abitrary [`OffsetDateTime`] ((between [min_time] and [max_time])),
    /// with an arbitrary time zone offset from UTC.
    pub fn arb_datetime_with_offset()
        (
            d in arb_datetime_in_range(min_time(), max_time()),
            off in arb_utc_offset(),
        ) -> OffsetDateTime {
            d.to_offset(off)
        }
}

prop_compose! {
    /// An abitrary [`OffsetDateTime`], offset in UTC, that can be represented
    /// as an RFC 3339 timestamp. Values with year 0 are further excluded
    /// due to the validity requirements on
    /// Google's well-known [`Timestamp`] protobuf message format.
    ///
    /// [`Timestamp`]: https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.Timestamp
    pub fn arb_protobuf_safe_datetime()
        (
            d in arb_datetime_in_range(
                min_protobuf_time(),
                max_protobuf_time(),
            )
        ) -> OffsetDateTime {
            d
        }
}

prop_compose! {
    fn arb_utc_offset_hms()
        (
            h in 0..=23i8,
            m in 0..=59i8,
            s in 0..=59i8,
        ) -> UtcOffset {
            UtcOffset::from_hms(h, m, s).unwrap()
        }
}

prop_compose! {
    /// An abitrary [`UtcOffset`].
    pub fn arb_utc_offset()
        (
            off in prop_oneof![
                Just(offset!(UTC)),
                arb_utc_offset_hms(),
                arb_utc_offset_hms().prop_map(|off| -off),
            ]
        ) -> UtcOffset {
            off
        }
}

// The following components of the timestamp follow
// Section 5.6 of RFC3339: https://tools.ietf.org/html/rfc3339#ref-ABNF.

prop_compose! {
    // See https://tools.ietf.org/html/rfc3339#appendix-A
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
    fn arb_rfc3339_day_of_year_and_month(year: i32, month: u8)
        (
            d in 1..num_days_in_month(year, month)
        ) -> u8 {
            d
        }
}

prop_compose! {
    fn arb_rfc3339_full_date()(year in 0..9999i32, month in 1..12u8)
        (
            day in arb_rfc3339_day_of_year_and_month(year, month),
            year in Just(year),
            month in Just(month),
        ) -> String {
            format!("{:0>4}-{:0>2}-{:0>2}", year, month, day)
        }
}

prop_compose! {
    /// An aribtrary RFC3339 timestamp
    ///
    /// For example: `1985-04-12T23:20:50.52Z`
    ///
    /// The implementaiton follows
    /// [Section 5.6 of RFC3339](https://tools.ietf.org/html/rfc3339#ref-ABNF)
    pub fn arb_rfc3339_timestamp()(
        date in arb_rfc3339_full_date(),
        time in arb_rfc3339_full_time()
    ) -> String {
        format!("{:}T{:}", date, time)
    }
}

/// Like `[arb_rfc3339_timestamp]`, but restricted to produce timestamps
/// that have a valid representation in
/// Google's well-known [`Timestamp`] protobuf message format.
///
/// [`Timestamp`]: https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.Timestamp
pub fn arb_protobuf_safe_rfc3339_timestamp() -> impl Strategy<Value = String> {
    arb_rfc3339_timestamp().prop_filter("timestamp out of protobuf range", |ts| {
        let t = OffsetDateTime::parse(ts, &Rfc3339).unwrap();
        (min_protobuf_time()..=max_protobuf_time()).contains(&t)
    })
}
