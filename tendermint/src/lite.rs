//! Core logic and traits of a light client.

pub mod error;
pub mod types;
pub mod verifier;

pub use self::types::*;

pub use self::verifier::{verify_bisection, verify_single, verify_single_inner};
