//! Provides [proptest](https://github.com/AltSysrq/proptest) generators for
//! time-like objects.

use std::convert::TryInto;

use chrono::{DateTime, NaiveDate, TimeZone, Timelike, Utc};
use proptest::prelude::*;

/// Any higher, and we're at seconds
pub const MAX_NANO_SECS: u32 = 999_999_999u32;

/// The most distant time in the past for which chrono produces correct
/// times from [Utc.timestamp](chrono::Utc.timestamp).
///
/// See <https://github.com/chronotope/chrono/issues/537>.
///
/// ```
/// use pbt_gen;
///
/// assert_eq!(pbt_gen::time::min_time().to_string(), "1653-02-10 06:13:21 UTC".to_string());
/// ```
pub fn min_time() -> DateTime<Utc> {
    Utc.timestamp(-9999999999, 0)
}

/// The most distant time in the future for which chrono produces correct
/// times from [Utc.timestamp](chrono::Utc.timestamp).
///
/// See <https://github.com/chronotope/chrono/issues/537>.
///
/// ```
/// use pbt_gen;
///
/// assert_eq!(pbt_gen::time::max_time().to_string(), "5138-11-16 09:46:39 UTC".to_string());
/// ```
pub fn max_time() -> DateTime<Utc> {
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
    /// An abitrary [chrono::DateTime] that is between the given `min`
    /// and `max`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{TimeZone, Utc};
    /// use pbt_gen;
    /// use proptest::prelude::*;
    ///
    /// proptest!{
    ///     fn rosa_luxemburg_and_octavia_butler_were_not_alive_at_the_same_time(
    ///        time_in_luxemburgs_lifespan in pbt_gen::time::arb_datetime_in_range(
    ///          Utc.ymd(1871, 3, 5).and_hms(0,0,0), // DOB
    ///          Utc.ymd(1919, 1, 15).and_hms(0,0,0), // DOD
    ///        ),
    ///        time_in_butlers_lifespan in pbt_gen::time::arb_datetime_in_range(
    ///          Utc.ymd(1947, 6, 22).and_hms(0,0,0), // DOB
    ///          Utc.ymd(2006, 2, 24).and_hms(0,0,0), // DOD
    ///        ),
    ///     ) {
    ///       prop_assert!(time_in_luxemburgs_lifespan != time_in_butlers_lifespan)
    ///     }
    /// }
    /// ```
    pub fn arb_datetime_in_range(min: DateTime<Utc>, max: DateTime<Utc>)(
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
    /// An abitrary [chrono::DateTime] (between [min_time] and [max_time]).
    pub fn arb_datetime()
        (
            d in arb_datetime_in_range(min_time(), max_time())
        ) -> DateTime<Utc> {
            d
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
