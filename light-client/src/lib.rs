#![forbid(unsafe_code)]
#![deny(
    // warnings,
    // missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms,
    nonstandard_style,
 )]

//! See the `light_client` module for the main documentation.

pub mod callback;
pub mod components;
pub mod contracts;
pub mod errors;
pub mod fork_detector;
pub mod light_client;
pub mod operations;
pub mod predicates;
pub mod prelude;
pub mod state;
pub mod store;
pub mod supervisor;
pub mod types;

mod macros;

#[doc(hidden)]
pub mod tests;
