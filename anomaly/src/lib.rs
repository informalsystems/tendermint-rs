//! **anomaly.rs**: Error context library with support for type-erased sources
//! and backtraces.
//!
//! # About
//!
//! **anomaly.rs** is an error library which provides support for concrete,
//! generic error types along with type-erased error sources/chains and
//! backtraces.
//!
//! Full support for all features is available on stable Rust.
//!
//! # Usage
//!
//! Below is some example boilerplate for how to define error types for your
//! project using **anomaly.rs**.
//!
//! This example uses the [`thiserror`] crate to provide custom derive support,
//! however it is not required: you can use any type which impls the standard
//! set of error traits (`Display`, `Debug`, and `std::error::Error`) as your
//! `ErrorKind` type:
//!
//! ```
//! use anomaly::{BoxError, Context};
//! use thiserror::Error;
//!
//! /// Use this type alias as your error type when returning `Result`.
//! ///
//! /// Internally it's defined as `Error(Box<Context<ErrorKind>>)` and
//! /// ensures propagation of errors is a cheap pointer copy.
//! pub type Error = anomaly::Error<ErrorKind>;
//!
//! /// An error kind type containing a domain-specific error categorization.
//! ///
//! /// You can convert this into the `Error` type above (and capture a
//! /// backtrace in the process) by using `.into()`.
//! ///
//! /// This example uses the `thiserror::Error` procedural macro to impl the
//! /// `Display` and `std::error::Error` traits, however you don't have to and
//! /// can impl them yourself or using another error crate of your choosing.
//! #[derive(Clone, Debug, Error)]
//! pub enum ErrorKind {
//!     /// See `thiserror` documentation for the `#[error]` attribute
//!     #[error("invalid argument: {name}")]
//!     Argument {
//!         name: String
//!     },
//!
//!     #[error("encoding error")]
//!     Encoding,
//!
//!     #[error("value overflowed")]
//!     Overflow
//! }
//!
//! impl ErrorKind {
//!    /// Add additional context (i.e. include a source error and capture
//!    /// a backtrace).
//!    ///
//!    /// You can convert the resulting `Context` into an `Error` by calling
//!    /// `.into()`.
//!    pub fn context(self, source: impl Into<BoxError>) -> Context<ErrorKind> {
//!        Context::new(self, Some(source.into()))
//!    }
//! }
//! ```
//!
//! # `serde` support
//!
//! The `serializer` Cargo feature of this crate enables [`serde`] serialization
//! support for errors, useful for structured logging of error kinds, their
//! backtraces, and their sources.
//!
//! When enabled, it adds a [`SerializedError`] type, as well as
//! [`serde::Serialize`] impls on the [`Context`] and [`Error`] types.
//!
//! These serializers preserve the full structure of your `ErrorKind`. For that
//! reasion, your `ErrorKind` must also impl [`serde::Serialize`].
//!
//! [`thiserror`]: https://github.com/dtolnay/thiserror

#![forbid(unsafe_code)]
#![allow(unused_attributes)]
#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]
#![doc(html_root_url = "https://docs.rs/anomaly/0.2.0")]
#[cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
#[macro_use]
mod macros;

mod context;
mod error;
mod message;
#[cfg(feature = "serializer")]
mod serializer;

#[cfg(feature = "serializer")]
pub use crate::serializer::SerializedError;
pub use crate::{context::Context, error::Error, message::Message};
#[cfg(feature = "backtrace")]
pub use backtrace;

/// Box containing a thread-safe + `'static` error suitable for use as a
/// as an `std::error::Error::source`.
// pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
