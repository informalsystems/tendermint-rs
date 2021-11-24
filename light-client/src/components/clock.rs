//! Provides an interface and a default implementation of the `Clock` component

use crate::types::Time;
/// Abstracts over the current time.
pub trait Clock: Send + Sync {
    /// Get the current time.
    fn now(&self) -> Time;
}

/// Provides the current wall clock time.
#[derive(Copy, Clone)]
pub struct SystemClock;

#[cfg(feature = "std")]
impl Clock for SystemClock {
    fn now(&self) -> Time {
        Time(chrono::Utc::now())
    }
}
