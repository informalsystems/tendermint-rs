//! JSONRPC IDs

use getrandom::getrandom;
use serde::{Deserialize, Serialize};

/// JSONRPC ID: request-specific identifier
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(untagged)]
pub enum Id {
    /// Numerical JSON ID
    Num(i64),
    /// String JSON ID
    Str(String),
    /// null JSON ID
    None,
}

impl Id {
    /// Create a JSONRPC ID containing a UUID v4 (i.e. random)
    pub fn uuid_v4() -> Self {
        let mut bytes = [0; 16];
        getrandom(&mut bytes).expect("RNG failure!");

        let uuid = uuid::Builder::from_bytes(bytes)
            .set_variant(uuid::Variant::RFC4122)
            .set_version(uuid::Version::Random)
            .build();

        Id::Str(uuid.to_string())
    }
}
