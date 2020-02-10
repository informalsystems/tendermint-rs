//! Timestamps

use crate::{
    error::Error,
    time::{ParseTimestamp, Time},
};
use chrono::{TimeZone, Utc};
use prost_amino_derive::Message;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, PartialEq, Message)]
pub struct TimeMsg {
    // TODO(ismail): switch to protobuf's well known type as soon as
    // https://github.com/tendermint/go-amino/pull/224 was merged
    // and tendermint caught up on the latest amino release.
    #[prost_amino(int64, tag = "1")]
    pub seconds: i64,
    #[prost_amino(int32, tag = "2")]
    pub nanos: i32,
}

impl ParseTimestamp for TimeMsg {
    fn parse_timestamp(&self) -> Result<Time, Error> {
        Ok(Utc.timestamp(self.seconds, self.nanos as u32).into())
    }
}

impl From<Time> for TimeMsg {
    fn from(ts: Time) -> TimeMsg {
        // TODO: non-panicking method for getting this?
        let duration = ts.duration_since(Time::unix_epoch()).unwrap();
        let seconds = duration.as_secs() as i64;
        let nanos = duration.subsec_nanos() as i32;

        TimeMsg { seconds, nanos }
    }
}

/// Converts `Time` to a `SystemTime`.
impl From<TimeMsg> for SystemTime {
    fn from(time: TimeMsg) -> SystemTime {
        if time.seconds >= 0 {
            UNIX_EPOCH + Duration::new(time.seconds as u64, time.nanos as u32)
        } else {
            UNIX_EPOCH - Duration::new(time.seconds as u64, time.nanos as u32)
        }
    }
}
