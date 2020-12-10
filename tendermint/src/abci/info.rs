use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// ABCI info
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Info(String);

impl AsRef<str> for Info {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for Info {
    fn default() -> Self {
        Self(String::new())
    }
}
