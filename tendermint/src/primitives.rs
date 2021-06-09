//! primitive
/// define String type in std and no_std
extern crate std as _std;
/// define time in std and no_std
#[cfg(feature = "std")]
pub use _std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(not(feature = "std"))]
pub use no_std_time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(not(feature = "std"))]
mod no_std_time {
    use chrono::TimeZone;
    use chrono::{DateTime, Utc};
    pub use core::time::Duration;
    use no_std_compat::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
    use no_std_compat::convert::TryFrom;
    use no_std_compat::ops::{Add, AddAssign, Sub, SubAssign};

    pub const UNIX_EPOCH: SystemTime = SystemTime { inner: 0.0 };

    #[derive(Debug, Copy, Clone)]
    pub struct SystemTime {
        /// Unit is milliseconds.
        inner: f64,
    }

    impl PartialEq for SystemTime {
        fn eq(&self, other: &SystemTime) -> bool {
            // Note that this will most likely only compare equal if we clone an `SystemTime`,
            // but that's ok.
            self.inner == other.inner
        }
    }

    impl Eq for SystemTime {}

    impl PartialOrd for SystemTime {
        fn partial_cmp(&self, other: &SystemTime) -> Option<Ordering> {
            self.inner.partial_cmp(&other.inner)
        }
    }

    impl Ord for SystemTime {
        fn cmp(&self, other: &Self) -> Ordering {
            self.inner.partial_cmp(&other.inner).unwrap()
        }
    }

    impl SystemTime {
        pub const UNIX_EPOCH: SystemTime = SystemTime { inner: 0.0 };

        pub fn now() -> SystemTime {
            let val = chrono::Utc::now();
            let val = val.timestamp() as f64;
            SystemTime { inner: val }
        }

        pub fn duration_since(&self, earlier: SystemTime) -> Result<Duration, ()> {
            let dur_ms = self.inner - earlier.inner;
            if dur_ms < 0.0 {
                return Err(());
            }
            Ok(Duration::from_millis(dur_ms as u64))
        }

        pub fn elapsed(&self) -> Result<Duration, ()> {
            self.duration_since(SystemTime::now())
        }

        pub fn checked_add(&self, duration: Duration) -> Option<SystemTime> {
            Some(*self + duration)
        }

        pub fn checked_sub(&self, duration: Duration) -> Option<SystemTime> {
            Some(*self - duration)
        }
    }

    impl Add<Duration> for SystemTime {
        type Output = SystemTime;

        fn add(self, other: Duration) -> SystemTime {
            let new_val = self.inner + other.as_millis() as f64;
            SystemTime { inner: new_val }
        }
    }

    impl Sub<Duration> for SystemTime {
        type Output = SystemTime;

        fn sub(self, other: Duration) -> SystemTime {
            let new_val = self.inner - other.as_millis() as f64;
            SystemTime { inner: new_val }
        }
    }

    impl AddAssign<Duration> for SystemTime {
        fn add_assign(&mut self, rhs: Duration) {
            *self = *self + rhs;
        }
    }

    impl SubAssign<Duration> for SystemTime {
        fn sub_assign(&mut self, rhs: Duration) {
            *self = *self - rhs;
        }
    }

    impl From<prost_types::Timestamp> for SystemTime {
        fn from(mut timestamp: prost_types::Timestamp) -> Self {
            timestamp.normalize();
            let system_time = if timestamp.seconds >= 0 {
                UNIX_EPOCH + Duration::from_secs(timestamp.seconds as u64)
            } else {
                UNIX_EPOCH - Duration::from_secs((-timestamp.seconds) as u64)
            };
            system_time + Duration::from_nanos(timestamp.nanos as u64)
        }
    }

    impl From<SystemTime> for prost_types::Timestamp {
        fn from(system_time: SystemTime) -> prost_types::Timestamp {
            let (seconds, nanos) = match system_time.duration_since(UNIX_EPOCH) {
                Ok(duration) => {
                    let seconds = i64::try_from(duration.as_secs()).unwrap();
                    (seconds, duration.subsec_nanos() as i32)
                }
                Err(_) => {
                    // Some maybe error
                    (1, 1_000_000_000)
                }
            };
            prost_types::Timestamp { seconds, nanos }
        }
    }

    impl From<DateTime<Utc>> for SystemTime {
        fn from(dt: DateTime<Utc>) -> Self {
            let sec = dt.timestamp();
            let nsec = dt.timestamp_subsec_nanos();
            if sec < 0 {
                // unlikely but should be handled
                UNIX_EPOCH - Duration::new(-sec as u64, 0) + Duration::new(0, nsec)
            } else {
                UNIX_EPOCH + Duration::new(sec as u64, nsec)
            }
        }
    }

    impl From<SystemTime> for DateTime<Utc> {
        fn from(t: SystemTime) -> DateTime<Utc> {
            let (sec, nsec) = match t.duration_since(UNIX_EPOCH) {
                Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
                Err(_) => {
                    // Some maybe error
                    (1, 1_000_000_000)
                }
            };
            Utc.timestamp(sec, nsec)
        }
    }
}
