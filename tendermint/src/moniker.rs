//! Monikers: names associated with validators

use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::{
    borrow::ToOwned,
    fmt::{self, Display},
    str::FromStr,
};
use alloc::string::String;

/// Validator display names
#[derive(Serialize, Deserialize, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Moniker(String);

impl FromStr for Moniker {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Moniker(s.to_owned()))
    }
}

impl AsRef<str> for Moniker {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for Moniker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
