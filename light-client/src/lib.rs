#![deny(rust_2018_idioms, nonstandard_style)]
#![warn(
     unreachable_pub,
     // missing_docs,
 )]

//! See the documentation of the `LightClient` struct.

pub mod components;
pub mod contracts;
pub mod errors;
pub mod light_client;
pub mod operations;
pub mod predicates;
pub mod prelude;
pub mod state;
pub mod store;
pub mod types;

mod macros;

#[doc(hidden)]
pub mod tests;
