#![no_std]

extern crate alloc;

mod prelude;

pub mod errors;
pub mod operations;
pub mod options;
pub mod predicates;
pub mod types;
mod verifier;

pub use verifier::{PredicateVerifier, Verdict, Verifier};

#[cfg(feature = "rust-crypto")]
pub use verifier::ProdVerifier;
