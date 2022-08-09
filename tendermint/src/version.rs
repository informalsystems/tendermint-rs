use core::fmt::{self, Debug, Display};

use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Tendermint version
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Version(pub String);

impl TryFrom<String> for Version {
    type Error = crate::error::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        // TODO(erwan): what validation does `Version` needs?
        Ok(Version(value))
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
