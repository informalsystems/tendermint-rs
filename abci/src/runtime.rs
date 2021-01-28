//! Abstractions for facilitating runtime-independent code.

#[cfg(feature = "blocking")]
pub mod blocking;
#[cfg(feature = "non-blocking")]
pub mod non_blocking;
