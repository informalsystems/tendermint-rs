//! Timestamps

use crate::{
    error::Error,
    time::{ParseTimestamp, Time},
};
use chrono::{TimeZone, Utc};
use prost_amino_derive::Message;
use std::convert::TryFrom;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, PartialEq, Message)]
pub struct Msg {
    // TODO(ismail): switch to protobuf's well known type as soon as
    // https://github.com/tendermint/go-amino/pull/224 was merged
    // and tendermint caught up on the latest amino release.
    #[prost_amino(int64, tag = "1")]
    pub seconds: i64,
    #[prost_amino(int32, tag = "2")]
    pub nanos: i32,
}

impl ParseTimestamp for Msg {
    fn parse_timestamp(&self) -> Result<Time, Error> {
        Ok(Utc.timestamp(self.seconds, self.nanos as u32).into())
    }
}

// TODO(xla): Convert to TryFrom.
#[allow(clippy::fallible_impl_from)]
impl From<Time> for Msg {
    fn from(ts: Time) -> Self {
        let duration = ts
            .duration_since(Time::unix_epoch())
            .expect("unable to get duration from epoch");
        let seconds = i64::try_from(duration.as_secs()).expect("overflow");
        let nanos = i32::try_from(duration.subsec_nanos()).expect("overflow");

        Self { seconds, nanos }
    }
}

/// Converts `Time` to a `SystemTime`.
impl From<Msg> for SystemTime {
    fn from(time: Msg) -> Self {
        if time.seconds >= 0 {
            UNIX_EPOCH + Duration::new(time.seconds as u64, time.nanos as u32)
        } else {
            UNIX_EPOCH - Duration::new(time.seconds as u64, time.nanos as u32)
        }
    }
}
