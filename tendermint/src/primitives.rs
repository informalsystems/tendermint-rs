//! primitive
/// define String type in std and no_std
#[cfg(feature = "std")]
pub use std::string::{String, ToString};

#[cfg(not(feature = "std"))]
pub use alloc::string::{String, ToString};

/// define time in std and no_std
#[cfg(feature = "std")]
pub use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(not(feature = "std"))]
pub use no_std_time::{Duration, SystemTime, UNIX_EPOCH};

/// define format macro in std and no_std
#[cfg(feature = "std")]
#[macro_export]
pub use std::format;

#[cfg(not(feature = "std"))]
pub use alloc::format;

#[cfg(not(feature = "std"))]
mod no_std_time {
    use sp_std::cmp::{Eq, PartialEq, Ord, PartialOrd, Ordering};
    use sp_std::ops::{Add, Sub, AddAssign, SubAssign};
    pub use core::time::Duration;

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
            let val = chrono::Date::now();
            SystemTime { inner: val }
        }

        pub fn duration_since(&self, earlier: SystemTime) -> Result<Duration, ()> {
            let dur_ms = self.inner - earlier.inner;
            if dur_ms < 0.0 {
                return Err(())
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
}
