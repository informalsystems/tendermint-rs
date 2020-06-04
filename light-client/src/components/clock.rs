use crate::prelude::*;
use dyn_clone::DynClone;

/// Abstracts over the current time.
pub trait Clock: Send + DynClone {
    /// Get the current time.
    fn now(&self) -> Time;
}

/// Provides the current wall clock time.
#[derive(Copy, Clone)]
pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> Time {
        Time::now()
    }
}
