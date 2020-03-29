use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display};

/// Tendermint version
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Version(String);

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
