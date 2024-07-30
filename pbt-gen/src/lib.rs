//! This internal crate provides [proptest](https://github.com/AltSysrq/proptest)
//! strategies and other PBT utilities used for testing.
//!
//! Conditions for inclusion in this crate are:
//!
//! 1. The utilities are relatively general.
//! 2. The utilities don't rely on any code internal to the other crates of this repository.
//!
//! The each module of this crate (and the module's dependencies) are guarded by
//! a feature, documented along with the module.
//!
//! The default features are:
//!
//! - "time"

/// Enabled with the "time" feature:
#[cfg(feature = "time")]
pub mod time;
