//! Provides an interface and a default implementation of the `Clock` component

use crate::types::Time;
use std::convert::TryInto;
use time::OffsetDateTime;

/// Abstracts over the current time.
pub trait Clock: Send + Sync {
    /// Get the current time.
    fn now(&self) -> Time;
}

/// Provides the current wall clock time.
#[derive(Copy, Clone)]
pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> Time {
        OffsetDateTime::now_utc()
            .try_into()
            .expect("system clock produces invalid time")
    }
}
