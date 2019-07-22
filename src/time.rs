//! Timestamps used by Tendermint blockchains

use crate::error::{Error, ErrorKind};
use chrono::{DateTime, Utc};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
#[cfg(feature = "tai64")]
use tai64::TAI64N;

/// Tendermint timestamps
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#time>
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Time(DateTime<Utc>);

impl Time {
    /// Get a `Timestamp` representing the current wall clock time
    pub fn now() -> Self {
        Time(Utc::now())
    }

    /// Get the `UNIX_EPOCH` time ("1970-01-01 00:00:00 UTC") as a `Timestamp`
    pub fn unix_epoch() -> Self {
        UNIX_EPOCH.into()
    }

    /// Calculate the amount of time which has passed since another `Timestamp`
    /// as a `std::time::Duration`
    pub fn duration_since(&self, other: Time) -> Result<Duration, Error> {
        self.0
            .signed_duration_since(other.0)
            .to_std()
            .map_err(|_| ErrorKind::OutOfRange.into())
    }

    /// Parse a timestamp from an RFC 3339 date
    pub fn parse_from_rfc3339(s: &str) -> Result<Time, Error> {
        Ok(Time(DateTime::parse_from_rfc3339(s)?.with_timezone(&Utc)))
    }

    /// Convert this timestamp to a `SystemTime`
    pub fn to_system_time(&self) -> Result<SystemTime, Error> {
        let duration_since_epoch = self.duration_since(Self::unix_epoch())?;
        Ok(UNIX_EPOCH + duration_since_epoch)
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
        t.to_system_time().unwrap()
    }
}

#[cfg(feature = "tai64")]
impl From<TAI64N> for Time {
    fn from(t: TAI64N) -> Time {
        Time(t.to_datetime_utc())
    }
}

#[cfg(feature = "tai64")]
impl From<Time> for TAI64N {
    fn from(t: Time) -> TAI64N {
        TAI64N::from_datetime_utc(&t.0)
    }
}

/// Parse `Timestamp` from a type
pub trait ParseTimestamp {
    /// Parse `Timestamp`, or return an `Error` if parsing failed
    fn parse_timestamp(&self) -> Result<Time, Error>;
}
