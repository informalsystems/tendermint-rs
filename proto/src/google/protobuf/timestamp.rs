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
