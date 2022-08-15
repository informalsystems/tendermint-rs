#![no_std]
#![allow(clippy::derive_partial_eq_without_eq)] // FIXME: Eq derivations

extern crate alloc;

mod prelude;

pub mod errors;
pub mod operations;
pub mod options;
pub mod predicates;
pub mod types;
mod verifier;

pub use verifier::{PredicateVerifier, ProdVerifier, Verdict, Verifier};
