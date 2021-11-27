use core::convert::TryInto;

use chrono::{DateTime, Duration, TimeZone, Utc};

use crate::google::protobuf as pb;

impl From<DateTime<Utc>> for pb::Timestamp {
    fn from(dt: DateTime<Utc>) -> pb::Timestamp {
        pb::Timestamp {
            seconds: dt.timestamp(),
            // This can exceed 1_000_000_000 in the case of a leap second, but
            // even with a leap second it should be under 2_147_483_647.
            nanos: dt
                .timestamp_subsec_nanos()
                .try_into()
                .expect("timestamp_subsec_nanos bigger than i32::MAX"),
        }
    }
}

impl From<pb::Timestamp> for DateTime<Utc> {
    fn from(ts: pb::Timestamp) -> DateTime<Utc> {
        Utc.timestamp(ts.seconds, ts.nanos as u32)
    }
}

// Note: we convert a protobuf::Duration into a chrono::Duration, not a
// std::time::Duration, because std::time::Durations are unsigned, but the
// protobuf duration is signed.

impl From<Duration> for pb::Duration {
    fn from(d: Duration) -> pb::Duration {
        // chrono's Duration stores the fractional part as `nanos: i32`
        // internally but doesn't provide a way to access it, only a way to get
        // the *total* number of nanoseconds. so we have to do this cool and fun
        // hoop-jumping maneuver
        let seconds = d.num_seconds();
        let nanos = (d - Duration::seconds(seconds))
            .num_nanoseconds()
            .expect("we computed the fractional part, so there's no overflow")
            .try_into()
            .expect("the fractional part fits in i32");

        pb::Duration { seconds, nanos }
    }
}

impl From<pb::Duration> for Duration {
    fn from(d: pb::Duration) -> Duration {
        // there's no constructor that supplies both at once
        Duration::seconds(d.seconds) + Duration::nanoseconds(d.nanos as i64)
    }
}
