//! LightNode
//!
//! Application based on the [Abscissa] framework.
//!
//! [Abscissa]: https://github.com/iqlusioninc/abscissa

// Tip: Deny warnings with `RUSTFLAGS="-D warnings"` environment variable in CI
#![recursion_limit = "256"]
#![forbid(unsafe_code)]
#![warn(
    rust_2018_idioms,
    trivial_casts,
    unused_lifetimes,
    unused_qualifications
)]
// TODO(ismail): add proper docs and remove this!
#![allow(missing_docs)]

#[macro_use]
extern crate futures;

pub mod application;
pub mod commands;
pub mod config;
pub mod error;
pub mod prelude;
pub mod requester;
pub mod store;
pub mod verifier;
