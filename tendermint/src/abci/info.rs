use crate::prelude::*;
use core::fmt::{self, Display};
use serde::{Deserialize, Serialize};

/// ABCI info
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
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
