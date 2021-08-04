//! Rust standard library types that can be fallibly cloned.

use std::net::TcpStream;

/// Types that can be cloned where success is not guaranteed can implement this
/// trait.
pub trait TryClone: Sized {
    /// The type of error that can be returned when an attempted clone
    /// operation fails.
    type Error;

    /// Attempt to clone this instance.
    ///
    /// # Errors
    /// Can fail if the underlying instance cannot be cloned (e.g. the OS could
    /// be out of file descriptors, or some low-level OS-specific error could
    /// be produced).
    fn try_clone(&self) -> Result<Self, Self::Error>;
}

impl TryClone for TcpStream {
    type Error = std::io::Error;

    fn try_clone(&self) -> Result<Self, Self::Error> {
        TcpStream::try_clone(self)
    }
}
