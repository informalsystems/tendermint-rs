use core::fmt::{self, Debug, Display};

use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Tendermint version
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Version(String);

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Version> for String {
    fn from(value: Version) -> Self {
        value.0
    }
}
