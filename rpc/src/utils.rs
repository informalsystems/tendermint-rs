//! Utility methods for the Tendermint RPC crate.

use rand::Rng;

use crate::prelude::*;

/// Produce a string containing a UUID.
///
/// Panics if random number generation fails.
pub fn uuid_str() -> String {
    let bytes: [u8; 16] = rand::thread_rng().gen();
    let uuid = uuid::Builder::from_random_bytes(bytes).into_uuid();
    uuid.to_string()
}
