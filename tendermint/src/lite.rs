//! Core logic and traits of a light client.

pub mod types;
pub mod verifier;

pub use self::types::*;

// TODO: don't expose this once the json tests
// switch to using one of the other functions
pub use self::verifier::verify_single;

pub use self::verifier::{verify_and_update_bisection, verify_and_update_single};
