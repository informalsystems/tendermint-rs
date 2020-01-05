use serde::{Deserialize, Serialize};

/// Channel IDs
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Id(pub u64);

impl Id {
    /// Get the current channel id as an integer
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl From<Id> for u64 {
    fn from(id: Id) -> Self {
        id.value()
    }
}

impl From<u64> for Id {
    fn from(id: u64) -> Self {
        Self(id)
    }
}
