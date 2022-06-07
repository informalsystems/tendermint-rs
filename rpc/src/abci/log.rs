use core::{fmt, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// ABCI log data
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Log(String);

impl Log {
    /// Access to the log message as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Log {
    fn from(s: &str) -> Self {
        Log(s.to_owned())
    }
}

impl AsRef<str> for Log {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
