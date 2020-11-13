use serde::{Deserialize, Serialize};

/// ABCI log data
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Log(String);

impl Log {
    /// constructor
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Convenience function: get value
    pub fn value(&self) -> &String {
        &self.0
    }
}
