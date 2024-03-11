//! Provides an interface and a default implementation of the `Clock` component

use time::OffsetDateTime;

use crate::verifier::types::Time;

/// Abstracts over the current time.
pub trait Clock: Send + Sync {
    /// Get the current time.
    fn now(&self) -> Time;
}

/// Provides the current wall clock time.
#[derive(Copy, Clone, Debug)]
pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> Time {
        OffsetDateTime::now_utc()
            .try_into()
            .expect("system clock produces invalid time")
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FixedClock {
    now: Time,
}

impl FixedClock {
    pub fn new(now: Time) -> Self {
        Self { now }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Time {
        self.now
    }
}
