use crate::{Error, ErrorKind};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// ABCI log data
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct Log(String);

impl Log {
    /// Parse the log data as JSON, returning a `serde_json::Value`
    pub fn parse_json(&self) -> Result<serde_json::Value, Error> {
        serde_json::from_str(&self.0).map_err(|_| ErrorKind::Parse.into())
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
