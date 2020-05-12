#![deny(rust_2018_idioms, nonstandard_style)]
#![warn(
     unreachable_pub,
     // missing_docs,
 )]

pub mod components;
pub mod errors;
pub mod macros;
pub mod operations;
pub mod predicates;
pub mod prelude;
pub mod store;
pub mod types;

#[doc(hidden)]
pub mod tests;

#[derive(Copy, Clone, Debug)]
pub enum Never {}
