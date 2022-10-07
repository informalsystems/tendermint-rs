//! ABCI response types used by the `/block_results` RPC endpoint.

use core::fmt::{self, Display};

use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Codespace
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Codespace(String);

impl AsRef<str> for Codespace {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for Codespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
