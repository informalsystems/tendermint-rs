//! Core logic and traits of a light client.

// TODO: don't expose the verifier, just the types and the public API.
// the verifier has low level functions that can be used incorrectly.

pub mod public;
pub mod types;
pub mod verifier;

pub use self::public::*;
pub use self::types::*;
pub use self::verifier::*;
