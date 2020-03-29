//! JSONRPC IDs

use getrandom::getrandom;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(untagged)]
pub enum IdType {
    Num(i64),
    Str(String),
    None,
}

/// JSONRPC ID: request-specific identifier
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Id(IdType);

impl Id {
    /// Create a JSONRPC ID containing a UUID v4 (i.e. random)
    pub fn uuid_v4() -> Self {
        let mut bytes = [0; 16];
        getrandom(&mut bytes).expect("RNG failure!");

        let uuid = uuid::Builder::from_bytes(bytes)
            .set_variant(uuid::Variant::RFC4122)
            .set_version(uuid::Version::Random)
            .build();

        Id(IdType::Str(uuid.to_string()))
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        match self {
            Id(IdType::Num(_)) => "",
            Id(IdType::Str(s)) => s.as_ref(),
            Id(IdType::None) => "",
        }
    }
}
