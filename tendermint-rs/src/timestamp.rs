//! Timestamps used by Tendermint blockchains

use crate::error::Error;
use chrono::{DateTime, Utc};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
#[cfg(feature = "tai64")]
use tai64::TAI64N;

/// Chain timestamps (e.g. consensus time)
#[cfg_attr(feature = "serializers", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Get a `Timestamp` representing the current wall clock time
    pub fn now() -> Self {
        Timestamp(Utc::now())
    }

    /// Get the `UNIX_EPOCH` time ("1970-01-01 00:00:00 UTC") as a `Timestamp`
    pub fn unix_epoch() -> Self {
        UNIX_EPOCH.into()
    }

    /// Calculate the amount of time which has passed since another `Timestamp`
    /// as a `std::time::Duration`
    pub fn duration_since(&self, other: Timestamp) -> Result<Duration, Error> {
        self.0
            .signed_duration_since(other.0)
            .to_std()
            .map_err(|_| Error::OutOfRange)
    }

    /// Parse a timestamp from an RFC 3339 date
    pub fn parse_from_rfc3339(s: &str) -> Result<Timestamp, Error> {
        Ok(Timestamp(
            DateTime::parse_from_rfc3339(s)?.with_timezone(&Utc),
        ))
    }

    /// Convert this timestamp to a `SystemTime`
    pub fn to_system_time(&self) -> Result<SystemTime, Error> {
        let duration_since_epoch = self.duration_since(Self::unix_epoch())?;
        Ok(UNIX_EPOCH + duration_since_epoch)
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(t: DateTime<Utc>) -> Timestamp {
        Timestamp(t)
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(t: Timestamp) -> DateTime<Utc> {
        t.0
    }
}

impl From<SystemTime> for Timestamp {
    fn from(t: SystemTime) -> Timestamp {
        Timestamp(t.into())
    }
}

impl From<Timestamp> for SystemTime {
    fn from(t: Timestamp) -> SystemTime {
        t.to_system_time().unwrap()
    }
}

#[cfg(feature = "tai64")]
impl From<TAI64N> for Timestamp {
    fn from(t: TAI64N) -> Timestamp {
        Timestamp(t.to_datetime_utc())
    }
}

#[cfg(feature = "tai64")]
impl From<Timestamp> for TAI64N {
    fn from(t: Timestamp) -> TAI64N {
        TAI64N::from_datetime_utc(&t.0)
    }
}

/// Parse `Timestamp` from a type
pub trait ParseTimestamp {
    /// Parse `Timestamp`, or return an `Error` if parsing failed
    fn parse_timestamp(&self) -> Result<Timestamp, Error>;
}
