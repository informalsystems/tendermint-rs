use crate::prelude::*;

/// Abstracts over the current time.
pub trait Clock {
    /// Get the current time.
    fn now(&self) -> Time;
}

/// Provides the current wall clock time.
pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> Time {
        Time::now()
    }
}
