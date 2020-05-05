use crate::prelude::*;

pub trait Clock {
    fn now(&self) -> Time;
}

pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> Time {
        Time::now()
    }
}
