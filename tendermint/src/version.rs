use crate::prelude::*;
use core::fmt::{self, Debug, Display};
use serde::{Deserialize, Serialize};

/// Tendermint version
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Version(String);

impl Version {
    pub fn unchecked(input: String) -> Self {
        Self(input)
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
