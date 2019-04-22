//! JSONRPC IDs

use rand_os::{rand_core::RngCore, OsRng};
use serde::{Deserialize, Serialize};

/// JSONRPC ID: request-specific identifier
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Id(String);

impl Id {
    /// Create a JSONRPC ID containing a UUID v4 (i.e. random)
    pub fn uuid_v4() -> Self {
        let mut rng = OsRng::new().unwrap();
        let mut bytes = [0; 16];
        rng.fill_bytes(&mut bytes);

        let uuid = uuid::Builder::from_bytes(bytes)
            .set_variant(uuid::Variant::RFC4122)
            .set_version(uuid::Version::Random)
            .build();

        Id(uuid.to_string())
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
